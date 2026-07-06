# -*- coding: utf-8 -*-
"""
menu.py
=======
Menü-Screens, die VOR bzw. rund um ein Spiel im pygame-Bereich erscheinen
(vergleichbar mit den Setup-Screens von Breakout/Tic-Tac-Toe):

- PreGameScreen : Auswahl Einzel-/Mehrspieler, Zugang zu den Optionen, Start.
- OptionsScreen : Sound/Lautstärke/Haptik + Tastenbelegung für beide Spieler
                  (jede Taste einzeln neu belegbar), plus Vorlagen.

Beide verhalten sich wie ein 'Game' (update/draw/handle_event), sind aber über
is_menu=True markiert, damit main.py sie von Pause/Highscore ausnimmt. Sie halten
eine Referenz auf die App und rufen deren Methoden (launch_game/show_screen/...).
"""

import pygame

import audio
import i18n
import settings as settings_mod
import ui
from game_base import Game, InputEvent
from i18n import t

# Farben kommen zentral aus dem UI-Toolkit (einheitlicher Look überall).
COL_BG = ui.BG_TOP
COL_PANEL = ui.PANEL
COL_SEL = ui.BTN_SEL
COL_BTN = ui.BTN
COL_TEXT = ui.TEXT
COL_MUTE = ui.TEXT_DIM
COL_ACCENT = ui.GREEN
COL_KEY = ui.GOLD

# Tkinter-keysym -> Übersetzungsschlüssel für gut lesbare Tastennamen.
_PRETTY = {
    "space": "key.space", "Return": "key.return", "Up": "key.up",
    "Down": "key.down", "Left": "key.left", "Right": "key.right",
    "Escape": "key.escape", "Tab": "key.tab",
}


def action_label(action):
    """Übersetztes Anzeige-Label einer Aktion (up/down/left/right/action)."""
    return t("action." + action)


def pretty_key(k):
    """Macht einen Tkinter-keysym gut lesbar (z.B. 'space' -> 'Leertaste')."""
    if not k:
        return t("key.none")
    if k in _PRETTY:
        return t(_PRETTY[k])
    return k.upper() if len(k) == 1 else k


class _Screen(Game):
    """Gemeinsame Basis für Menü-Screens (keine Spiel-Logik/Highscore)."""

    is_menu = True
    highscore_key = "_menu"

    def __init__(self, surface, width, height, app):
        self.app = app
        # Menüs nutzen die globalen App-Einstellungen direkt (Änderungen wirken).
        super().__init__(surface, width, height, mode="single",
                         game_settings=app.settings)

    def reset(self):
        self.score = 0
        self.game_over = False

    def update(self, dt):
        pass


# ---------------------------------------------------------------------------
#  Vorspiel-Screen: Modus wählen / Optionen / Start
# ---------------------------------------------------------------------------

class PreGameScreen(_Screen):
    name = "Auswahl"

    def __init__(self, surface, width, height, app, game_cls):
        self.game_cls = game_cls
        super().__init__(surface, width, height, app)
        self.name = t("pregame.name")
        self._build_buttons()
        self.sel = 0

    def _build_buttons(self):
        self.buttons = []   # (label, callback)
        # Spiel-eigene Modi (z.B. Invaders: Klassik/Arena). Definiert ein Spiel
        # eine Liste MODES = [(mode_key, i18n_label), ...], werden diese statt
        # Einzel-/Mehrspieler angeboten.
        modes = getattr(self.game_cls, "MODES", None)
        if modes:
            for mode_key, label_key in modes:
                self.buttons.append(
                    (t(label_key),
                     lambda m=mode_key: self.app.launch_game(self.game_cls, m)))
        else:
            self.buttons.append((t("pregame.single"),
                                 lambda: self.app.launch_game(self.game_cls, "single")))
            if getattr(self.game_cls, "supports_multiplayer", False):
                self.buttons.append((t("pregame.multi"),
                                     lambda: self.app.launch_game(self.game_cls, "multi")))
        self.buttons.append((t("pregame.options"), self._open_options))
        self.buttons.append((t("pregame.back"), self.app.back_to_menu))

        # Rechtecke für Maus/Anzeige berechnen (zentriert, gestapelt).
        # Kleinere Buttons + Start unterhalb des Untertitels ("Modus wählen"),
        # damit dieser immer sichtbar bleibt (auch bei mehreren Modi).
        self.rects = []
        bw, bh, gap = 300, 40, 10
        total = len(self.buttons) * (bh + gap) - gap
        y0 = max(132, self.height // 2 - total // 2 + 6)
        for i in range(len(self.buttons)):
            x = self.width // 2 - bw // 2
            y = y0 + i * (bh + gap)
            self.rects.append(pygame.Rect(x, y, bw, bh))

    def _open_options(self):
        opts = OptionsScreen(self.surface, self.width, self.height, self.app,
                             on_close=self._reopen)
        self.app.show_screen(opts)

    def _reopen(self):
        self.app.show_screen(PreGameScreen(self.surface, self.width, self.height,
                                           self.app, self.game_cls))

    def handle_event(self, event):
        if event.kind == InputEvent.KEYDOWN:
            if event.key == "Escape":
                self.app.back_to_menu()
            elif self.is_action(event.key, "up") or event.key == "Up":
                self.sel = (self.sel - 1) % len(self.buttons)
                self.play_sound("move")
            elif self.is_action(event.key, "down") or event.key == "Down":
                self.sel = (self.sel + 1) % len(self.buttons)
                self.play_sound("move")
            elif event.key in ("Return", "space"):
                self._activate(self.sel)
        elif event.kind == InputEvent.MOUSEMOVE:
            for i, r in enumerate(self.rects):
                if r.collidepoint(event.pos):
                    self.sel = i
        elif event.kind == InputEvent.MOUSEDOWN:
            for i, r in enumerate(self.rects):
                if r.collidepoint(event.pos):
                    self._activate(i)

    def _activate(self, i):
        self.play_sound("click")
        self.buttons[i][1]()

    def draw(self):
        s = self.surface
        ui.draw_background(s, self.width, self.height)

        ui.draw_title(s, self.width, self.game_cls.name,
                      subtitle=t("pregame.mode"), y=64)

        btn_font = ui.font(19)
        for i, (label, _) in enumerate(self.buttons):
            ui.draw_button(s, self.rects[i], label, btn_font,
                           selected=(i == self.sel))

        ui.draw_footer(s, self.width, self.height, t("pregame.hint"))


# ---------------------------------------------------------------------------
#  Options-Screen: Sound/Haptik + Tastenbelegung
# ---------------------------------------------------------------------------

class OptionsScreen(_Screen):
    name = "Optionen"

    def __init__(self, surface, width, height, app, on_close):
        self.on_close = on_close
        super().__init__(surface, width, height, app)
        self.name = t("options.name")
        # Etwas kompaktere Schrift: bei 640px Breite brauchen zwei Spalten
        # inkl. langer Werte ("640 x 480 (Standard)") sonst zu viel Platz.
        self.font = ui.font(18, mono=True)
        self.capture = None       # (player, action), während eine Taste neu belegt wird
        self.preset_idx = 0
        # Aktuelle Auswahl für Auflösung/FPS aus den Einstellungen ableiten.
        self.res_idx = settings_mod.resolution_index(self.settings.get("resolution"))
        self.fps_idx = settings_mod.fps_index(self.settings.get("fps", 60))
        self._build_items()
        self.sel = 0

    def on_surface_changed(self):
        """Wird von der App nach einer Auflösungsänderung gerufen (Layout neu)."""
        self._build_items()
        if self.sel >= len(self.items):
            self.sel = len(self.items) - 1

    def _build_items(self):
        """Baut die interaktive Liste inkl. Zeichenpositionen auf (an Größe angepasst)."""
        self.items = []
        W, H = self.width, self.height
        # Zwei Spalten, deren Position sich an der Breite orientiert -> passt auch
        # bei kleinen Auflösungen.
        self._left_x = 40
        self._right_x = W // 2 + 10
        col_w = min(300, W // 2 - 50)

        def add(kind, rect, **kw):
            it = dict(kind=kind, rect=rect)
            it.update(kw)
            self.items.append(it)
            return it

        # Linke Spalte oben: Ton/Steuerungs-Vorlage
        y = 70
        for kind, kw in (("toggle", dict(key="sound", label=t("options.sound"))),
                         ("volume", dict(label=t("options.volume"))),
                         ("toggle", dict(key="haptik", label=t("options.haptik"))),
                         ("preset", dict(label=t("options.preset")))):
            add(kind, pygame.Rect(self._left_x, y, col_w, 30), **kw)
            y += 38

        # Rechte Spalte oben: Grafik & Leistung (Auto / Auflösung / FPS / Sprache)
        ry = 70
        add("toggle", pygame.Rect(self._right_x, ry, col_w, 30),
            key="auto_resolution", label=t("options.auto_res"))
        ry += 38
        add("resolution", pygame.Rect(self._right_x, ry, col_w, 30),
            label=t("options.resolution"))
        ry += 38
        add("fps", pygame.Rect(self._right_x, ry, col_w, 30), label=t("options.fps"))
        ry += 38
        add("language", pygame.Rect(self._right_x, ry, col_w, 30),
            label=t("options.language"))

        # Steuerungs-Spalten für Spieler 1 (links) und Spieler 2 (rechts)
        for player, px in (("p1", self._left_x), ("p2", self._right_x)):
            y = 262
            for act in settings_mod.ACTIONS:
                add("bind", pygame.Rect(px, y, col_w, 26), player=player, action=act,
                    label=action_label(act))
                y += 30

        # Schliessen-Button unten
        add("button", pygame.Rect(W // 2 - 190, H - 44, 380, 34),
            label=t("options.save_back"), on_activate=self._close)

        # Karten-Panels hinter den vier Gruppen berechnen (Audio, Grafik,
        # Steuerung P1/P2) - rein optisch, aus den Item-Rechtecken abgeleitet.
        n_act = len(settings_mod.ACTIONS)
        groups = (self.items[0:4], self.items[4:8],
                  self.items[8:8 + n_act], self.items[8 + n_act:8 + 2 * n_act])
        self._panels = []
        for grp in groups:
            if not grp:
                continue
            u = grp[0]["rect"].copy()
            for it in grp[1:]:
                u.union_ip(it["rect"])
            self._panels.append(u.inflate(28, 24))

    def _close(self):
        settings_mod.save_settings(self.settings)
        self.on_close()

    # ----- Eingabe ------------------------------------------------------

    def handle_event(self, event):
        if event.kind == InputEvent.KEYDOWN:
            # Im Belege-Modus fängt die nächste Taste die neue Belegung ab.
            if self.capture is not None:
                self._capture_key(event.key)
                return
            self._key_nav(event.key)
        elif event.kind == InputEvent.MOUSEMOVE:
            for i, it in enumerate(self.items):
                if it["rect"].collidepoint(event.pos):
                    self.sel = i
        elif event.kind == InputEvent.MOUSEDOWN:
            for i, it in enumerate(self.items):
                if it["rect"].collidepoint(event.pos):
                    self.sel = i
                    self._click_item(it, event.pos)
                    break

    def _click_item(self, it, pos):
        """Maus-Klick: Pfeile < > getrennt auswerten, Lautstärke-Balken direkt setzen."""
        kind = it["kind"]
        if kind in ("bind", "button"):
            self._activate(it)
        elif kind == "volume":
            bar = it.get("bar_rect")
            if bar and bar.w:
                v = (pos[0] - bar.x) / bar.w
                self.settings["volume"] = max(0.0, min(1.0, round(v, 2)))
                self._save_and_beep()
            else:
                self._adjust(it, +1)
        else:
            # < value >  -> Klick auf den linken Pfeil verkleinert, rechter vergrößert.
            dec, inc = it.get("dec_rect"), it.get("inc_rect")
            if dec and dec.collidepoint(pos):
                self._adjust(it, -1)
            elif inc and inc.collidepoint(pos):
                self._adjust(it, +1)
            elif kind == "toggle":
                self._adjust(it, +1)   # Umschalter: Klick irgendwo schaltet um
            # sonst: Klick nur auf das Label -> keine Änderung

    def _capture_key(self, key):
        player, action = self.capture
        if key != "Escape":
            self.settings["controls"][player][action] = key
            settings_mod.save_settings(self.settings)
            self.play_sound("select")
        self.capture = None

    def _key_nav(self, key):
        if key == "Escape":
            self._close()
        elif key in ("Up", "w"):
            self.sel = (self.sel - 1) % len(self.items)
            self.play_sound("move")
        elif key in ("Down", "s"):
            self.sel = (self.sel + 1) % len(self.items)
            self.play_sound("move")
        elif key in ("Left", "a"):
            self._adjust(self.items[self.sel], -1)
        elif key in ("Right", "d"):
            self._adjust(self.items[self.sel], +1)
        elif key in ("Return", "space"):
            self._activate(self.items[self.sel])

    def _adjust(self, it, direction):
        """Links/Rechts: Toggles umschalten, Lautstärke/Vorlage ändern."""
        if it["kind"] == "toggle":
            self.settings[it["key"]] = not self.settings.get(it["key"], False)
            if it["key"] == "auto_resolution":
                # Sofort anwenden (an Fenster anpassen bzw. feste Auflösung zurück).
                self.app.set_auto_resolution(self.settings["auto_resolution"])
            self._save_and_beep()
        elif it["kind"] == "volume":
            v = self.settings.get("volume", 0.6) + 0.1 * direction
            self.settings["volume"] = max(0.0, min(1.0, round(v, 2)))
            self._save_and_beep()
        elif it["kind"] == "preset":
            self.preset_idx = (self.preset_idx + direction) % len(settings_mod.PRESETS)
            settings_mod.apply_preset(self.settings, self.preset_idx)
            self.controls = self.settings["controls"]
            self._save_and_beep()
        elif it["kind"] == "resolution":
            # Im Auto-Modus wird die Auflösung vom Fenster bestimmt -> nicht manuell.
            if self.settings.get("auto_resolution"):
                return
            self.res_idx = (self.res_idx + direction) % len(settings_mod.RESOLUTIONS)
            w, h = settings_mod.RESOLUTIONS[self.res_idx][1]
            # Wendet die Auflösung sofort an und baut dieses Menü neu auf.
            self.app.apply_resolution(w, h)
            settings_mod.save_settings(self.settings)
            self.play_sound("select")
        elif it["kind"] == "fps":
            self.fps_idx = (self.fps_idx + direction) % len(settings_mod.FPS_OPTIONS)
            self.app.apply_fps(settings_mod.FPS_OPTIONS[self.fps_idx])
            self._save_and_beep()
        elif it["kind"] == "language":
            codes = [c for c, _ in i18n.AVAILABLE]
            cur = codes.index(i18n.get_language()) if i18n.get_language() in codes else 0
            i18n.set_language(codes[(cur + direction) % len(codes)])
            self.app.refresh_language()   # Tkinter-Menü neu beschriften
            self.name = t("options.name")
            self._build_items()           # Labels dieses Screens neu übersetzen
            self.play_sound("select")

    def _activate(self, it):
        """Enter/Klick: Belegen starten, Button auslösen oder Toggle schalten."""
        if it["kind"] == "bind":
            self.capture = (it["player"], it["action"])
            self.play_sound("click")
        elif it["kind"] == "button":
            self.play_sound("click")
            it["on_activate"]()
        else:
            self._adjust(it, +1)

    def _save_and_beep(self):
        settings_mod.save_settings(self.settings)
        self.play_sound("select")

    # ----- Zeichnen -----------------------------------------------------

    def draw(self):
        s = self.surface
        ui.draw_background(s, self.width, self.height, stars=False)

        # Karten-Panels hinter den Gruppen (rein optisch).
        for p in getattr(self, "_panels", ()):
            ui.draw_panel(s, p, radius=10, shadow=False)

        # Titel oben links mit Akzent-Unterstrich.
        title_font = ui.font(22, bold=True)
        title = title_font.render(t("options.title"), True, COL_TEXT)
        s.blit(title, (self._left_x, 16))
        pygame.draw.rect(s, ui.ACCENT,
                         (self._left_x, 18 + title.get_height(),
                          title.get_width(), 3), border_radius=2)

        # Abschnitts-Überschriften (klein, Akzentfarbe).
        head_font = ui.font(15, bold=True)
        s.blit(head_font.render(t("options.graphics"), True, ui.ACCENT),
               (self._right_x, 38))
        s.blit(head_font.render(t("common.player1"), True, ui.ACCENT),
               (self._left_x, 232))
        s.blit(head_font.render(t("common.player2"), True, ui.ACCENT),
               (self._right_x, 232))

        for i, it in enumerate(self.items):
            r = it["rect"]
            if i == self.sel and it["kind"] != "button":
                hl = r.inflate(16, 8)
                pygame.draw.rect(s, ui.PANEL_LIGHT, hl, border_radius=6)
                pygame.draw.rect(s, ui.ACCENT, (hl.x, hl.y + 3, 3, hl.h - 6),
                                 border_radius=2)
            self._draw_item(it, selected=(i == self.sel))

        if self.capture is not None:
            self._draw_capture_overlay()

    def _fit_render(self, text, color, max_w):
        """Rendert Text; wird er breiter als max_w, in kleineren Stufen erneut."""
        img = self.font.render(text, True, color)
        for size in (16, 14, 12):
            if img.get_width() <= max_w:
                break
            img = ui.font(size, mono=True).render(text, True, color)
        return img

    def _draw_arrow_value(self, it, farbe, value_text, value_col):
        """Zeichnet 'Label            < Wert >' mit einzeln anklickbaren Pfeilen.

        Speichert die Trefferflächen der Pfeile in it['dec_rect']/it['inc_rect'].
        """
        s = self.surface
        r = it["rect"]
        label = self.font.render(it["label"], True, farbe)
        s.blit(label, (r.x, r.y))

        lt = self.font.render("<", True, farbe)
        gt = self.font.render(">", True, farbe)
        # Wert notfalls kleiner rendern, damit er nicht ins Label läuft.
        max_w = r.w - label.get_width() - lt.get_width() - gt.get_width() - 34
        val = self._fit_render(str(value_text), value_col, max_w)
        pad = 10
        gx = r.right - gt.get_width()
        vx = gx - pad - val.get_width()
        lx = vx - pad - lt.get_width()
        s.blit(lt, (lx, r.y))
        s.blit(val, (vx, r.y))
        s.blit(gt, (gx, r.y))
        # Etwas größere Trefferflächen für bequemes Klicken.
        it["dec_rect"] = pygame.Rect(lx - 6, r.y - 4, lt.get_width() + 12, r.height + 8)
        it["inc_rect"] = pygame.Rect(gx - 6, r.y - 4, gt.get_width() + 12, r.height + 8)

    def _draw_item(self, it, selected):
        s = self.surface
        r = it["rect"]
        farbe = COL_TEXT if selected else COL_MUTE
        kind = it["kind"]

        if kind == "toggle":
            an = self.settings.get(it["key"], False)
            wert = t("common.on") if an else t("common.off")
            self._draw_arrow_value(it, farbe, wert, COL_ACCENT if an else COL_MUTE)

        elif kind == "language":
            self._draw_arrow_value(it, farbe, dict(i18n.AVAILABLE)[i18n.get_language()],
                                   COL_KEY)

        elif kind == "volume":
            s.blit(self.font.render(it["label"], True, farbe), (r.x, r.y))
            v = self.settings.get("volume", 0.6)
            # Prozentwert rechtsbündig, Balken links davon -> bleibt im Panel.
            pct = self.font.render(f"{int(v * 100)}%", True, farbe)
            bar = pygame.Rect(r.right - pct.get_width() - 8 - 110, r.y + 8, 110, 12)
            it["bar_rect"] = bar
            pygame.draw.rect(s, COL_BTN, bar, border_radius=6)
            fill_w = int(bar.w * v)
            if fill_w > 0:
                pygame.draw.rect(s, ui.ACCENT,
                                 (bar.x, bar.y, fill_w, bar.h), border_radius=6)
            pygame.draw.rect(s, ui.BORDER_LIGHT, bar, width=1, border_radius=6)
            # Griff-Knopf am Ende des Füllstands
            knob_x = bar.x + max(4, min(bar.w - 4, fill_w))
            pygame.draw.circle(s, COL_TEXT, (knob_x, bar.centery), 5)
            s.blit(pct, (r.right - pct.get_width(), r.y))

        elif kind == "preset":
            self._draw_arrow_value(it, farbe,
                                   t(settings_mod.PRESETS[self.preset_idx][0]), COL_KEY)

        elif kind == "resolution":
            if self.settings.get("auto_resolution"):
                # Auto: keine Pfeile, aktuelle Fenster-Auflösung anzeigen.
                it["dec_rect"] = it["inc_rect"] = None
                label = self.font.render(it["label"], True, farbe)
                s.blit(label, (r.x, r.y))
                txt = t("options.res_auto", w=self.width, h=self.height)
                img = self._fit_render(txt, COL_MUTE,
                                       r.w - label.get_width() - 12)
                s.blit(img, (r.right - img.get_width(), r.y))
            else:
                self._draw_arrow_value(it, farbe,
                                       t(settings_mod.RESOLUTIONS[self.res_idx][0]),
                                       COL_KEY)

        elif kind == "fps":
            self._draw_arrow_value(it, farbe,
                                   str(settings_mod.FPS_OPTIONS[self.fps_idx]), COL_ACCENT)

        elif kind == "bind":
            s.blit(self.font.render(it["label"], True, farbe), (r.x, r.y))
            key = self.key_for(it["player"], it["action"])
            img = self.font.render(pretty_key(key), True, COL_KEY)
            # Tastenkappen-Chip hinter dem Tastennamen
            chip = pygame.Rect(0, 0, img.get_width() + 14, img.get_height() + 4)
            chip.midright = (r.right, r.centery)
            pygame.draw.rect(s, ui.PANEL_LIGHT if selected else COL_BTN, chip,
                             border_radius=5)
            pygame.draw.rect(s, ui.BORDER_LIGHT, chip, width=1, border_radius=5)
            s.blit(img, img.get_rect(center=chip.center))

        elif kind == "button":
            ui.draw_button(s, r, it["label"], self.font, selected=selected)

    def _draw_capture_overlay(self):
        s = self.surface
        overlay = pygame.Surface((self.width, self.height), pygame.SRCALPHA)
        overlay.fill((8, 10, 18, 185))
        s.blit(overlay, (0, 0))

        # Zentrierte Karte mit pulsierendem Akzentrahmen ("wartet auf Taste").
        cw, ch = min(420, self.width - 40), 170
        card = pygame.Rect(self.width // 2 - cw // 2,
                           self.height // 2 - ch // 2, cw, ch)
        ui.draw_panel(s, card, radius=14)
        glow = int(120 + 100 * ui.pulse(2.6))
        pygame.draw.rect(s, (ui.ACCENT[0], ui.ACCENT[1], ui.ACCENT[2]),
                         card, width=2, border_radius=14)
        pygame.draw.rect(s, (glow // 3, glow // 2, glow),
                         card.inflate(8, 8), width=1, border_radius=16)

        player, action = self.capture
        who = t("common.player1") if player == "p1" else t("common.player2")
        self.draw_center_text(f"{who}  -  {action_label(action)}",
                              self.font, ui.ACCENT, -44)
        self.draw_center_text(t("options.press_key"),
                              ui.font(30, bold=True), COL_TEXT, 0)
        self.draw_center_text(t("options.cancel"), self.font, COL_MUTE, 46)


# ---------------------------------------------------------------------------
#  Sprachauswahl-Screen (beim ersten Start; zweisprachige Beschriftung)
# ---------------------------------------------------------------------------

class LanguageScreen(_Screen):
    name = "Sprache"

    def __init__(self, surface, width, height, app, on_done):
        self.on_done = on_done
        super().__init__(surface, width, height, app)
        self.name = t("lang.name")
        self._build()
        codes = [c for c, _ in i18n.AVAILABLE]
        self.sel = codes.index(i18n.get_language()) if i18n.get_language() in codes else 0

    def _build(self):
        self.rects = []
        bw, bh, gap = 300, 56, 16
        total = len(i18n.AVAILABLE) * (bh + gap) - gap
        y0 = self.height // 2 - total // 2
        for i in range(len(i18n.AVAILABLE)):
            x = self.width // 2 - bw // 2
            self.rects.append(pygame.Rect(x, y0 + i * (bh + gap), bw, bh))

    def on_surface_changed(self):
        self._build()

    def _choose(self, i):
        i18n.set_language(i18n.AVAILABLE[i][0])
        self.app.refresh_language()
        self.play_sound("click")
        self.on_done()

    def handle_event(self, event):
        if event.kind == InputEvent.KEYDOWN:
            if event.key in ("Up", "w", "Left", "a"):
                self.sel = (self.sel - 1) % len(self.rects)
                self.play_sound("move")
            elif event.key in ("Down", "s", "Right", "d"):
                self.sel = (self.sel + 1) % len(self.rects)
                self.play_sound("move")
            elif event.key in ("Return", "space"):
                self._choose(self.sel)
        elif event.kind == InputEvent.MOUSEMOVE:
            for i, r in enumerate(self.rects):
                if r.collidepoint(event.pos):
                    self.sel = i
        elif event.kind == InputEvent.MOUSEDOWN:
            for i, r in enumerate(self.rects):
                if r.collidepoint(event.pos):
                    self._choose(i)

    def draw(self):
        s = self.surface
        ui.draw_background(s, self.width, self.height)
        ui.draw_title(s, self.width, t("lang.title"), y=80,
                      big=ui.font(34, bold=True))
        btn_font = ui.font(24, bold=True)
        code_font = ui.font(13)
        for i, (code, label) in enumerate(i18n.AVAILABLE):
            ui.draw_button(s, self.rects[i], label, btn_font,
                           selected=(i == self.sel),
                           sub=code.upper(), sub_font=code_font)
        ui.draw_footer(s, self.width, self.height, t("lang.hint"))
