# -*- coding: utf-8 -*-
"""
profileview.py
==============
Die Oberfläche zu profiles.py:

- ``ProfileScreen``    : Profile anlegen, umbenennen, umgestalten, löschen und
                         wechseln. Jede Karte zeigt Name, Symbol, Farbe,
                         Level/Rang, Punkte und Spielzeit.
- ``PlayerPickScreen`` : kleine Auswahl für Spieler 2 vor einer
                         Mehrspieler-Partie (Profile + "Gast").
- ``LevelIntroScreen`` : einmalige Karte "Dein Startlevel" beim ersten Start
                         nach dem Erfolgs-Update.

Die Screens sind ganz normale ``menu._Screen``-Ableitungen und werden über
``App.show_screen`` angezeigt.
"""

import pygame

import achievements
import levels
import profiles
import stats
import ui
from game_base import InputEvent
from i18n import t
from menu import _Screen

_ROW_H = 62
_GAP = 8


class ProfileScreen(_Screen):
    """Profilverwaltung: wer spielt, und wer bekommt die Punkte?"""

    name = "Profile"

    def __init__(self, surface, width, height, app, on_close):
        self.on_close = on_close
        self.sel = 0
        self.editor = None       # None oder dict(pid, name-Eingabe, icon, color)
        self._msg = ""
        self._buttons = []
        super().__init__(surface, width, height, app)
        self.name = t("profile.name")
        self._build()

    # ----- Aufbau -------------------------------------------------------

    def on_surface_changed(self):
        self._build()

    def _build(self):
        W, H = self.width, self.height
        self._left = max(28, W // 2 - 300)
        self._right = W - self._left
        self._top = 108
        self._bottom = H - 56
        self._list = profiles.all_profiles()
        self._stats = {pid: stats.totals(pid) for pid, _m in self._list}
        self.sel = max(0, min(self.sel, len(self._list)))

    # ----- Eingabe ------------------------------------------------------

    def handle_event(self, event):
        if self.editor is not None:
            self._editor_event(event)
            return
        if event.kind == InputEvent.KEYDOWN:
            key = event.key
            if key == "Escape":
                self.on_close()
            elif key in ("Up", "w"):
                self.sel = (self.sel - 1) % (len(self._list) + 1)
                self.play_sound("move")
            elif key in ("Down", "s"):
                self.sel = (self.sel + 1) % (len(self._list) + 1)
                self.play_sound("move")
            elif key in ("Return", "space"):
                self._activate(self.sel)
            elif key == "n":
                self._new_profile()
            elif key == "F2" and self.sel < len(self._list):
                self._edit_profile(self._list[self.sel][0])
            elif key == "Delete" and self.sel < len(self._list):
                self._delete_profile(self._list[self.sel][0])
        elif event.kind == InputEvent.MOUSEMOVE:
            for i, (r, _cb) in enumerate(self._buttons):
                if r.collidepoint(event.pos):
                    self.sel = i if i <= len(self._list) else self.sel
        elif event.kind == InputEvent.MOUSEDOWN:
            for r, cb in self._buttons:
                if r.collidepoint(event.pos):
                    self.play_sound("click")
                    cb()
                    return

    def _activate(self, index):
        if index >= len(self._list):
            self._new_profile()
            return
        self._switch(self._list[index][0])

    def _switch(self, pid):
        if pid == profiles.active_id():
            self.on_close()
            return
        self.play_sound("click")
        profiles.switch(pid)
        self.app.refresh_profile()
        self._build()
        self._msg = t("profile.switched", name=profiles.name_of(pid))

    def _new_profile(self):
        if profiles.count() >= 8:
            self._msg = t("profile.err_full")
            return
        self.editor = {"pid": None, "field": ui.TextInput(maxlen=profiles.MAX_NAME,
                                                          placeholder=t("profile.name_hint")),
                       "icon": 0, "color": profiles.count() % len(profiles.COLORS),
                       "err": ""}

    def _edit_profile(self, pid):
        meta = profiles.meta(pid)
        icon = profiles.icon_of(pid)
        self.editor = {"pid": pid,
                       "field": ui.TextInput(text=meta.get("name", ""),
                                             maxlen=profiles.MAX_NAME,
                                             placeholder=t("profile.name_hint")),
                       "icon": profiles.ICONS.index(icon) if icon in profiles.ICONS else 0,
                       "color": meta.get("color", 0), "err": ""}

    def _delete_profile(self, pid):
        if profiles.count() <= 1:
            self._msg = t("profile.err_last")
            return
        if profiles.delete(pid):
            self.app.refresh_profile()
            self._build()
            self._msg = t("profile.deleted")

    # -- Editor -----------------------------------------------------------

    def _editor_event(self, event):
        ed = self.editor
        if event.kind == InputEvent.KEYDOWN:
            if event.key == "Escape":
                self.editor = None
                return
            if event.key in ("Return", "KP_Enter"):
                self._save_editor()
                return
            if event.key in ("Left", "Right") and not ed["field"].text:
                step = -1 if event.key == "Left" else 1
                ed["color"] = (ed["color"] + step) % len(profiles.COLORS)
                return
            ed["field"].handle(event)
        elif event.kind == InputEvent.MOUSEDOWN:
            for r, cb in self._buttons:
                if r.collidepoint(event.pos):
                    self.play_sound("click")
                    cb()
                    return

    def _save_editor(self):
        ed = self.editor
        name = ed["field"].text.strip()
        icon = profiles.ICONS[ed["icon"] % len(profiles.ICONS)]
        if ed["pid"] is None:
            pid, err = profiles.create(name, icon=icon, color=ed["color"])
            if err:
                ed["err"] = t("profile.err_" + err)
                return
            profiles.switch(pid)
        else:
            err = profiles.rename(ed["pid"], name)
            if err:
                ed["err"] = t("profile.err_" + err)
                return
            profiles.set_look(ed["pid"], icon=icon, color=ed["color"])
        self.editor = None
        self.app.refresh_profile()
        self._build()

    # ----- Zeichnen -----------------------------------------------------

    def draw(self):
        s = self.surface
        W, H = self.width, self.height
        self._buttons = []
        ui.draw_background(s, W, H, stars=False)
        ui.draw_title(s, W, t("profile.name"), subtitle=t("profile.subtitle"),
                      y=40, big=ui.font(28, bold=True), accent=ui.ACCENT)

        y = self._top
        for i, (pid, _meta) in enumerate(self._list):
            self._draw_row(s, pid, y, i)
            y += _ROW_H + _GAP

        # "Neues Profil" als letzte Zeile.
        rect = pygame.Rect(self._left, y, self._right - self._left, 34)
        selected = (self.sel == len(self._list))
        ui.draw_panel(s, rect, color=ui.PANEL_LIGHT if selected else ui.PANEL,
                      shadow=False, radius=10)
        pygame.draw.rect(s, ui.ACCENT if selected else ui.BORDER, rect, 1,
                         border_radius=10)
        label = ui.font(13, bold=True).render(
            "+  " + t("profile.new"), True,
            ui.TEXT if selected else ui.TEXT_DIM)
        s.blit(label, label.get_rect(center=rect.center))
        self._buttons.append((rect, self._new_profile))

        if self._msg:
            img = ui.font(12).render(self._msg, True, ui.GOLD)
            s.blit(img, (self._left, self._bottom + 4))

        if self.editor is not None:
            self._draw_editor(s, W, H)
            ui.draw_footer(s, W, H, t("profile.hint_edit"))
        else:
            ui.draw_footer(s, W, H, t("profile.hint"))

    def _draw_row(self, s, pid, y, index):
        s_rect = pygame.Rect(self._left, y, self._right - self._left, _ROW_H)
        active = (pid == profiles.active_id())
        selected = (index == self.sel)
        ui.draw_panel(s, s_rect,
                      color=ui.PANEL_LIGHT if (selected or active) else ui.PANEL,
                      shadow=False, radius=12)
        color = profiles.color_rgb(pid)
        if active:
            pygame.draw.rect(s, color, s_rect, 2, border_radius=12)
        achievements.draw_badge(s, (s_rect.x + 34, s_rect.centery), 20,
                                profiles.icon_of(pid), color)

        name = ui.font(17, bold=True).render(profiles.name_of(pid), True,
                                             ui.TEXT)
        s.blit(name, (s_rect.x + 64, s_rect.y + 9))
        tot = self._stats.get(pid, {})
        level = tot.get("level", 1)
        sub = ui.font(11).render(
            "%s %d · %s · %s · %s" % (
                t("level.short"), level, levels.rank_title(level),
                t("progress.points", n=tot.get("points", 0)),
                stats.format_time(tot.get("time", 0))),
            True, ui.TEXT_DIM)
        s.blit(sub, (s_rect.x + 64, s_rect.y + 12 + name.get_height()),
               (0, 0, s_rect.w - 190, sub.get_height()))

        # Aktionen rechts: Wechseln (nur wenn nicht aktiv), Bearbeiten, Löschen.
        x = s_rect.right - 12
        fnt = ui.font(11, bold=True)
        for label, cb, danger in (
                ("✕", lambda p=pid: self._delete_profile(p), True),
                ("✎", lambda p=pid: self._edit_profile(p), False),
                (t("profile.play") if not active else t("profile.active"),
                 lambda p=pid: self._switch(p), False)):
            w = max(26, fnt.size(label)[0] + 16)
            r = pygame.Rect(x - w, s_rect.centery - 11, w, 22)
            fill = ui.PANEL if not danger else ui.mix(ui.PANEL, ui.RED, 0.25)
            pygame.draw.rect(s, fill, r, border_radius=11)
            pygame.draw.rect(s, ui.BORDER_LIGHT, r, 1, border_radius=11)
            img = fnt.render(label, True, ui.TEXT_DIM)
            s.blit(img, img.get_rect(center=r.center))
            self._buttons.append((r, cb))
            x -= w + 6

    def _draw_editor(self, s, W, H):
        ed = self.editor
        overlay = pygame.Surface((W, H), pygame.SRCALPHA)
        overlay.fill((0, 0, 0, 150))
        s.blit(overlay, (0, 0))

        bw, bh = min(460, W - 60), 250
        box = pygame.Rect(W // 2 - bw // 2, H // 2 - bh // 2, bw, bh)
        ui.draw_panel(s, box, color=ui.PANEL_LIGHT, radius=14,
                      accent_top=ui.ACCENT)
        title = ui.font(16, bold=True).render(
            t("profile.new") if ed["pid"] is None else t("profile.edit"),
            True, ui.TEXT)
        s.blit(title, (box.x + 18, box.y + 12))

        # Namensfeld.
        field = pygame.Rect(box.x + 18, box.y + 46, box.w - 36, 30)
        ed["field"].draw(s, field, ui.font(15), focused=True)

        # Symbol-Auswahl.
        label = ui.font(11).render(t("profile.icon"), True, ui.TEXT_DIM)
        s.blit(label, (box.x + 18, field.bottom + 8))
        x = box.x + 18
        y = field.bottom + 24
        for i, icon in enumerate(profiles.ICONS):
            r = pygame.Rect(x, y, 30, 30)
            chosen = (i == ed["icon"])
            col = profiles.COLORS[ed["color"] % len(profiles.COLORS)]
            rgb = tuple(int(col.lstrip("#")[k:k + 2], 16) for k in (0, 2, 4))
            achievements.draw_badge(s, r.center, 13, icon,
                                    rgb if chosen else ui.BORDER_LIGHT)
            if chosen:
                pygame.draw.circle(s, ui.TEXT, r.center, 15, 1)
            self._buttons.append((r, lambda k=i: ed.__setitem__("icon", k)))
            x += 34
            if x + 30 > box.right - 18:
                x = box.x + 18
                y += 34

        # Farb-Auswahl.
        label = ui.font(11).render(t("profile.color"), True, ui.TEXT_DIM)
        s.blit(label, (box.x + 18, y + 34))
        x = box.x + 18
        cy = y + 52
        for i, col in enumerate(profiles.COLORS):
            rgb = tuple(int(col.lstrip("#")[k:k + 2], 16) for k in (0, 2, 4))
            r = pygame.Rect(x, cy, 22, 22)
            pygame.draw.rect(s, rgb, r, border_radius=11)
            if i == ed["color"]:
                pygame.draw.rect(s, ui.TEXT, r.inflate(6, 6), 1,
                                 border_radius=14)
            self._buttons.append((r, lambda k=i: ed.__setitem__("color", k)))
            x += 28

        if ed["err"]:
            img = ui.font(11, bold=True).render(ed["err"], True, ui.RED)
            s.blit(img, (box.x + 18, box.bottom - 30))

        # Speichern / Abbrechen.
        fnt = ui.font(12, bold=True)
        x = box.right - 18
        for label_key, cb in (("profile.cancel", lambda: setattr(self, "editor", None)),
                              ("profile.save", self._save_editor)):
            label = t(label_key)
            w = fnt.size(label)[0] + 24
            r = pygame.Rect(x - w, box.bottom - 38, w, 26)
            pygame.draw.rect(s, ui.BTN_SEL if label_key == "profile.save"
                             else ui.BTN, r, border_radius=13)
            img = fnt.render(label, True, ui.TEXT)
            s.blit(img, img.get_rect(center=r.center))
            self._buttons.append((r, cb))
            x -= w + 8


class PlayerPickScreen(_Screen):
    """Auswahl des Profils für Spieler 2 vor einer Mehrspieler-Partie."""

    name = "Spieler 2"

    def __init__(self, surface, width, height, app, on_pick, on_cancel,
                 game_cls=None):
        self.on_pick = on_pick
        self.on_cancel = on_cancel
        self.game_cls = game_cls
        self.sel = 0
        self._rects = []
        super().__init__(surface, width, height, app)
        self.name = t("profile.pick_name")
        self._build()

    def on_surface_changed(self):
        self._build()

    def _build(self):
        # Alle Profile außer dem aktiven (das ist Spieler 1) + Gast.
        me = profiles.active_id()
        self.options = [(pid, profiles.name_of(pid))
                        for pid, _m in profiles.all_profiles() if pid != me]
        self.options.append((profiles.GUEST, t("profile.guest")))

    def handle_event(self, event):
        if event.kind == InputEvent.KEYDOWN:
            if event.key == "Escape":
                self.on_cancel()
            elif event.key in ("Up", "w"):
                self.sel = (self.sel - 1) % len(self.options)
                self.play_sound("move")
            elif event.key in ("Down", "s"):
                self.sel = (self.sel + 1) % len(self.options)
                self.play_sound("move")
            elif event.key in ("Return", "space"):
                self._pick(self.sel)
        elif event.kind == InputEvent.MOUSEMOVE:
            for i, r in enumerate(self._rects):
                if r.collidepoint(event.pos):
                    self.sel = i
        elif event.kind == InputEvent.MOUSEDOWN:
            for i, r in enumerate(self._rects):
                if r.collidepoint(event.pos):
                    self._pick(i)
                    return

    def _pick(self, index):
        self.play_sound("click")
        pid, _name = self.options[index]
        self.on_pick(None if pid == profiles.GUEST else pid)

    def draw(self):
        s = self.surface
        W, H = self.width, self.height
        ui.draw_background(s, W, H, stars=False)
        accent = (ui.game_color(self.game_cls.__name__)
                  if self.game_cls else ui.ACCENT)
        ui.draw_title(s, W, t("profile.pick_name"),
                      subtitle=t("profile.pick_sub",
                                 name=profiles.name_of()), y=64, accent=accent)

        bw = min(320, W - 60)
        bh = 40
        gap = 8
        total = len(self.options) * (bh + gap) - gap
        y0 = max(140, H // 2 - total // 2)
        self._rects = []
        for i, (pid, name) in enumerate(self.options):
            r = pygame.Rect(W // 2 - bw // 2, y0 + i * (bh + gap), bw, bh)
            self._rects.append(r)
            selected = (i == self.sel)
            ui.draw_panel(s, r, color=ui.PANEL_LIGHT if selected else ui.PANEL,
                          shadow=False, radius=12)
            col = (profiles.color_rgb(pid) if pid != profiles.GUEST
                   else ui.TEXT_FAINT)
            if selected:
                pygame.draw.rect(s, col, r, 2, border_radius=12)
            achievements.draw_badge(s, (r.x + 24, r.centery), 13,
                                    profiles.icon_of(pid), col)
            img = ui.font(15, bold=True).render(name, True,
                                                ui.TEXT if selected else ui.TEXT_DIM)
            s.blit(img, img.get_rect(midleft=(r.x + 46, r.centery)))
            if pid != profiles.GUEST:
                level, _pts = profiles.level_of(pid)
                lvl = ui.font(11).render("%s %d" % (t("level.short"), level),
                                         True, ui.TEXT_FAINT)
                s.blit(lvl, lvl.get_rect(midright=(r.right - 14, r.centery)))
        ui.draw_footer(s, W, H, t("profile.pick_hint"))


class LevelIntroScreen(_Screen):
    """Einmalige Begrüßung nach dem Update: "Dein Startlevel"."""

    name = "Level"

    def __init__(self, surface, width, height, app, on_close):
        self.on_close = on_close
        super().__init__(surface, width, height, app)
        self.name = t("level.intro_name")
        self._rarest = achievements.rarest_unlocked(3)
        self._level, self._points = stats.level_of()
        self._n_unlocked, self._n_total = achievements.counts()

    def handle_event(self, event):
        if event.kind == InputEvent.KEYDOWN and event.key in (
                "Escape", "Return", "space", "KP_Enter"):
            self.on_close()
        elif event.kind == InputEvent.MOUSEDOWN:
            self.on_close()

    def draw(self):
        s = self.surface
        W, H = self.width, self.height
        ui.draw_background(s, W, H)
        color = levels.rank_color(self._level)
        ui.draw_title(s, W, t("level.intro_title"),
                      subtitle=t("level.intro_sub"), y=70, accent=color)

        bw, bh = min(520, W - 60), min(300, H - 200)
        box = pygame.Rect(W // 2 - bw // 2, 150, bw, bh)
        ui.draw_panel(s, box, color=ui.PANEL_LIGHT, radius=16, accent_top=color)

        achievements.draw_badge(s, (box.centerx, box.y + 52), 34, "bolt", color)
        big = ui.font(28, bold=True).render(
            "%s %d" % (t("level.short"), self._level), True, ui.TEXT)
        s.blit(big, big.get_rect(midtop=(box.centerx, box.y + 92)))
        rank = ui.font(15, bold=True).render(levels.rank_title(self._level),
                                             True, color)
        s.blit(rank, rank.get_rect(midtop=(box.centerx, box.y + 92 + big.get_height())))

        sub = ui.font(12).render(
            t("level.intro_points", n=self._points, a=self._n_unlocked,
              m=self._n_total), True, ui.TEXT_DIM)
        s.blit(sub, sub.get_rect(midtop=(box.centerx, box.y + 130 + big.get_height())))

        y = box.y + 160 + big.get_height()
        for d, ts in self._rarest:
            achievements.draw_badge(s, (box.x + 40, y + 12), 12, d["icon"],
                                    achievements.rarity_color(d))
            img = ui.font(12, bold=True).render(achievements.display_name(d),
                                                True, ui.TEXT)
            s.blit(img, (box.x + 60, y + 2))
            meta = ui.font(10).render(
                "%s · %s" % (achievements.rarity_name(d), str(ts)[:10]),
                True, ui.TEXT_FAINT)
            s.blit(meta, (box.x + 60, y + 4 + img.get_height()))
            y += 30
        ui.draw_footer(s, W, H, t("level.intro_hint"))
