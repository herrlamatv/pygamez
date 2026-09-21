/**
 * profileview.js - Profilverwaltung der Web-Version (Port von profileview.py).
 *
 * Profile anlegen, umbenennen, umgestalten, löschen und wechseln. Jede Karte
 * zeigt Name, Symbol, Farbe, Level/Rang, Punkte und Spielzeit. Dazu die
 * einmalige Karte "Dein Startlevel" nach dem Erfolgs-Update.
 */
(function () {
  "use strict";

  const { ui, t } = PG;
  const W = PG.W;
  const H = PG.H;
  const ROW_H = 62;
  const GAP = 8;

  class ProfileScreen {
    constructor(app, onClose) {
      this.app = app;
      this.isMenu = true;
      this.gameOver = false;
      this.paused = false;
      this.onClose = onClose || (() => app.backToMenu());
      this.name = t("profile.name");
      this.sel = 0;
      this.editor = null;
      this.msg = "";
      this.buttons = [];
      this.build();
    }

    build() {
      this.left = Math.max(28, W / 2 - 300);
      this.right = W - this.left;
      this.top = 108;
      this.bottomY = H - 56;
      this.list = PG.profiles.all();
      this.stats = {};
      for (const p of this.list) this.stats[p.id] = PG.stats.totals(p.id);
      this.sel = Math.max(0, Math.min(this.sel, this.list.length));
    }

    // ----- Eingabe --------------------------------------------------------
    handleEvent(e) {
      if (this.editor) return this.editorEvent(e);
      if (e.kind === "keydown") {
        const k = e.key;
        if (k === "Escape") this.onClose();
        else if (k === "Up" || k === "w") {
          this.sel = PG.mod(this.sel - 1, this.list.length + 1);
          PG.audio.play("move");
        } else if (k === "Down" || k === "s") {
          this.sel = PG.mod(this.sel + 1, this.list.length + 1);
          PG.audio.play("move");
        } else if (k === "Return" || k === "space" || k === "KP_Enter") {
          this.activate(this.sel);
        } else if (k === "n" || k === "N") this.newProfile();
        else if (k === "F2" && this.sel < this.list.length) this.editProfile(this.list[this.sel].id);
        else if (k === "Delete" && this.sel < this.list.length) this.deleteProfile(this.list[this.sel].id);
      } else if (e.kind === "mousedown" && e.button === 1) {
        for (const [r, cb] of this.buttons) {
          if (r.collidepoint(e.pos)) {
            PG.audio.play("click");
            cb();
            return;
          }
        }
      }
    }

    editorEvent(e) {
      const ed = this.editor;
      if (e.kind === "keydown") {
        if (e.key === "Escape") {
          this.editor = null;
          return;
        }
        if (e.key === "Return" || e.key === "KP_Enter") {
          this.saveEditor();
          return;
        }
        if ((e.key === "Left" || e.key === "Right") && !ed.field.text) {
          ed.color = PG.mod(ed.color + (e.key === "Left" ? -1 : 1), PG.profiles.COLORS.length);
          return;
        }
        ed.field.handle(e);
      } else if (e.kind === "mousedown" && e.button === 1) {
        for (const [r, cb] of this.buttons) {
          if (r.collidepoint(e.pos)) {
            PG.audio.play("click");
            cb();
            return;
          }
        }
      }
    }

    activate(index) {
      if (index >= this.list.length) this.newProfile();
      else this.switchTo(this.list[index].id);
    }

    switchTo(pid) {
      if (pid === PG.profiles.activeId()) {
        this.onClose();
        return;
      }
      PG.profiles.switchTo(pid);
      this.app.refreshProfile();
      this.build();
      this.msg = t("profile.switched", { name: PG.profiles.nameOf(pid) });
    }

    newProfile() {
      if (PG.profiles.count() >= PG.profiles.MAX) {
        this.msg = t("profile.err_full");
        return;
      }
      this.editor = {
        pid: null,
        field: new ui.TextInput("", PG.profiles.MAX_NAME, null, t("profile.name_hint")),
        icon: 0,
        color: PG.profiles.count() % PG.profiles.COLORS.length,
        err: "",
      };
    }

    editProfile(pid) {
      const meta = PG.profiles.meta(pid);
      const icon = PG.profiles.iconOf(pid);
      this.editor = {
        pid,
        field: new ui.TextInput(meta.name || "", PG.profiles.MAX_NAME, null, t("profile.name_hint")),
        icon: Math.max(0, PG.profiles.ICONS.indexOf(icon)),
        color: meta.color || 0,
        err: "",
      };
    }

    deleteProfile(pid) {
      if (PG.profiles.count() <= 1) {
        this.msg = t("profile.err_last");
        return;
      }
      if (PG.profiles.del(pid)) {
        this.app.refreshProfile();
        this.build();
        this.msg = t("profile.deleted");
      }
    }

    saveEditor() {
      const ed = this.editor;
      const name = ed.field.text.trim();
      const icon = PG.profiles.ICONS[ed.icon % PG.profiles.ICONS.length];
      if (ed.pid == null) {
        const [pid, err] = PG.profiles.create(name, icon, ed.color);
        if (err) {
          ed.err = t("profile.err_" + err);
          return;
        }
        PG.profiles.switchTo(pid);
      } else {
        const err = PG.profiles.rename(ed.pid, name);
        if (err) {
          ed.err = t("profile.err_" + err);
          return;
        }
        PG.profiles.setLook(ed.pid, icon, ed.color);
      }
      this.editor = null;
      this.app.refreshProfile();
      this.build();
    }

    update() {}

    // ----- Zeichnen -------------------------------------------------------
    draw(ctx) {
      this.buttons = [];
      ui.drawBackground(ctx, W, H, false);
      ui.drawTitle(ctx, W, t("profile.name"), {
        subtitle: t("profile.subtitle"), y: 40, big: ui.font(28, true), accent: ui.ACCENT,
      });

      let y = this.top;
      this.list.forEach((p, i) => {
        this.drawRow(ctx, p, y, i);
        y += ROW_H + GAP;
      });

      const rect = new PG.Rect(this.left, y, this.right - this.left, 34);
      const selected = this.sel === this.list.length;
      ui.drawPanel(ctx, rect, { color: selected ? ui.PANEL_LIGHT : ui.PANEL, shadow: false, radius: 10 });
      PG.draw.rect(ctx, selected ? ui.ACCENT : ui.BORDER, rect, 1, 10);
      ui.text(ctx, "+  " + t("profile.new"), rect.centerx, rect.centery, ui.font(13, true),
        selected ? ui.TEXT : ui.TEXT_DIM, "center");
      this.buttons.push([rect, () => this.newProfile()]);

      if (this.msg) ui.text(ctx, this.msg, this.left, this.bottomY + 4, ui.font(12), ui.GOLD);

      if (this.editor) {
        this.drawEditor(ctx);
        ui.drawFooter(ctx, W, H, t("profile.hint_edit"));
      } else {
        ui.drawFooter(ctx, W, H, t("profile.hint"));
      }
    }

    drawRow(ctx, p, y, index) {
      const row = new PG.Rect(this.left, y, this.right - this.left, ROW_H);
      const active = p.id === PG.profiles.activeId();
      const selected = index === this.sel;
      ui.drawPanel(ctx, row, { color: selected || active ? ui.PANEL_LIGHT : ui.PANEL, shadow: false, radius: 12 });
      const color = ui.hexToRgb(PG.profiles.colorHex(p.id));
      if (active) PG.draw.rect(ctx, color, row, 2, 12);
      PG.ach.drawBadge(ctx, [row.x + 34, row.centery], 20, PG.profiles.iconOf(p.id), color, false);

      const nameFont = ui.font(17, true);
      ui.text(ctx, PG.profiles.nameOf(p.id), row.x + 64, row.y + 9, nameFont, ui.TEXT);
      const tot = this.stats[p.id] || {};
      const level = tot.level || 1;
      ui.text(ctx, `${t("level.short")} ${level} · ${PG.ach.rankTitle(level)} · ${t("progress.points", { n: tot.points || 0 })} · ${PG.stats.formatTime(tot.time || 0)}`,
        row.x + 64, row.y + 12 + nameFont.height, ui.font(11), ui.TEXT_DIM);

      let x = row.right - 12;
      const fnt = ui.font(11, true);
      const actions = [
        ["✕", () => this.deleteProfile(p.id), true],
        ["✎", () => this.editProfile(p.id), false],
        [active ? t("profile.active") : t("profile.play"), () => this.switchTo(p.id), false],
      ];
      for (const [label, cb, danger] of actions) {
        const w = Math.max(26, fnt.width(label) + 16);
        const r = new PG.Rect(x - w, row.centery - 11, w, 22);
        PG.draw.rect(ctx, danger ? ui.mix(ui.PANEL, ui.RED, 0.25) : ui.PANEL, r, 0, 11);
        PG.draw.rect(ctx, ui.BORDER_LIGHT, r, 1, 11);
        ui.text(ctx, label, r.centerx, r.centery, fnt, ui.TEXT_DIM, "center");
        this.buttons.push([r, cb]);
        x -= w + 6;
      }
    }

    drawEditor(ctx) {
      const ed = this.editor;
      PG.draw.rect(ctx, [0, 0, 0, 150], [0, 0, W, H]);
      const bw = Math.min(460, W - 60);
      const bh = 250;
      const box = new PG.Rect(W / 2 - bw / 2, H / 2 - bh / 2, bw, bh);
      ui.drawPanel(ctx, box, { color: ui.PANEL_LIGHT, radius: 14, accentTop: ui.ACCENT });
      ui.text(ctx, ed.pid == null ? t("profile.new") : t("profile.edit"),
        box.x + 18, box.y + 12, ui.font(16, true), ui.TEXT);

      const field = new PG.Rect(box.x + 18, box.y + 46, box.w - 36, 30);
      ed.field.draw(ctx, field, ui.font(15), true);

      ui.text(ctx, t("profile.icon"), box.x + 18, field.bottom + 8, ui.font(11), ui.TEXT_DIM);
      let x = box.x + 18;
      let y = field.bottom + 24;
      const rgb = ui.hexToRgb(PG.profiles.COLORS[ed.color % PG.profiles.COLORS.length]);
      PG.profiles.ICONS.forEach((icon, i) => {
        const r = new PG.Rect(x, y, 30, 30);
        const chosen = i === ed.icon;
        PG.ach.drawBadge(ctx, [r.centerx, r.centery], 13, icon, chosen ? rgb : ui.BORDER_LIGHT, false);
        if (chosen) PG.draw.circle(ctx, ui.TEXT, [r.centerx, r.centery], 15, 1);
        this.buttons.push([r, () => {
          ed.icon = i;
        }]);
        x += 34;
        if (x + 30 > box.right - 18) {
          x = box.x + 18;
          y += 34;
        }
      });

      ui.text(ctx, t("profile.color"), box.x + 18, y + 34, ui.font(11), ui.TEXT_DIM);
      x = box.x + 18;
      const cy = y + 52;
      PG.profiles.COLORS.forEach((hex, i) => {
        const r = new PG.Rect(x, cy, 22, 22);
        PG.draw.rect(ctx, ui.hexToRgb(hex), r, 0, 11);
        if (i === ed.color) PG.draw.rect(ctx, ui.TEXT, r.inflate(6, 6), 1, 14);
        this.buttons.push([r, () => {
          ed.color = i;
        }]);
        x += 28;
      });

      if (ed.err) ui.text(ctx, ed.err, box.x + 18, box.bottom - 30, ui.font(11, true), ui.RED);

      const fnt = ui.font(12, true);
      let bx = box.right - 18;
      const actions = [
        [t("profile.cancel"), () => {
          this.editor = null;
        }, false],
        [t("profile.save"), () => this.saveEditor(), true],
      ];
      for (const [label, cb, primary] of actions) {
        const w = fnt.width(label) + 24;
        const r = new PG.Rect(bx - w, box.bottom - 38, w, 26);
        PG.draw.rect(ctx, primary ? ui.BTN_SEL : ui.BTN, r, 0, 13);
        ui.text(ctx, label, r.centerx, r.centery, fnt, ui.TEXT, "center");
        this.buttons.push([r, cb]);
        bx -= w + 8;
      }
    }
  }

  /** Einmalige Begrüßung nach dem Update: "Dein Startlevel". */
  class LevelIntroScreen {
    constructor(app, onClose) {
      this.app = app;
      this.isMenu = true;
      this.gameOver = false;
      this.paused = false;
      this.onClose = onClose || (() => app.backToMenu());
      this.name = t("level.intro_name");
      this.rarest = PG.ach.rarestUnlocked(3);
      [this.level, this.points] = PG.stats.levelOf();
      [this.nUnlocked, this.nTotal] = PG.ach.counts();
    }

    handleEvent(e) {
      if ((e.kind === "keydown" && ["Escape", "Return", "space", "KP_Enter"].includes(e.key))
        || (e.kind === "mousedown" && e.button === 1)) {
        this.onClose();
      }
    }

    update() {}

    draw(ctx) {
      ui.drawBackground(ctx, W, H);
      const color = PG.ach.rankColor(this.level);
      ui.drawTitle(ctx, W, t("level.intro_title"), { subtitle: t("level.intro_sub"), y: 70, accent: color });

      const bw = Math.min(520, W - 60);
      const bh = Math.min(300, H - 200);
      const box = new PG.Rect(W / 2 - bw / 2, 150, bw, bh);
      ui.drawPanel(ctx, box, { color: ui.PANEL_LIGHT, radius: 16, accentTop: color });

      PG.ach.drawBadge(ctx, [box.centerx, box.y + 52], 34, "bolt", color, false);
      const bigFont = ui.font(28, true);
      ui.text(ctx, `${t("level.short")} ${this.level}`, box.centerx, box.y + 92, bigFont, ui.TEXT, "midtop");
      ui.text(ctx, PG.ach.rankTitle(this.level), box.centerx, box.y + 92 + bigFont.height,
        ui.font(15, true), color, "midtop");
      ui.text(ctx, t("level.intro_points", { n: this.points, a: this.nUnlocked, m: this.nTotal }),
        box.centerx, box.y + 130 + bigFont.height, ui.font(12), ui.TEXT_DIM, "midtop");

      let y = box.y + 160 + bigFont.height;
      for (const [d, ts] of this.rarest) {
        PG.ach.drawBadge(ctx, [box.x + 40, y + 12], 12, d.icon, PG.ach.rarityColor(d), false);
        const f = ui.font(12, true);
        ui.text(ctx, PG.ach.displayName(d), box.x + 60, y + 2, f, ui.TEXT);
        ui.text(ctx, `${PG.ach.rarityName(d)} · ${String(ts).slice(0, 10)}`,
          box.x + 60, y + 4 + f.height, ui.font(10), ui.TEXT_FAINT);
        y += 30;
      }
      ui.drawFooter(ctx, W, H, t("level.intro_hint"));
    }
  }

  PG.ProfileScreen = ProfileScreen;
  PG.LevelIntroScreen = LevelIntroScreen;
})();
