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
from game_base import Game, InputEvent
from i18n import t

COL_BG = (18, 20, 28)
COL_PANEL = (30, 34, 46)
COL_SEL = (70, 96, 150)
COL_BTN = (44, 50, 66)
COL_TEXT = (232, 234, 240)
COL_MUTE = (150, 158, 176)
COL_ACCENT = (120, 200, 140)
COL_KEY = (240, 210, 120)

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
        self.rects = []
        bw, bh, gap = 380, 50, 14
        total = len(self.buttons) * (bh + gap) - gap
        y0 = self.height // 2 - total // 2 + 20
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
        s.fill(COL_BG)

        title = self.big_font.render(self.game_cls.name, True, COL_TEXT)
        s.blit(title, title.get_rect(center=(self.width // 2, 70)))
        sub = self.font.render(t("pregame.mode"), True, COL_MUTE)
        s.blit(sub, sub.get_rect(center=(self.width // 2, 112)))

        for i, (label, _) in enumerate(self.buttons):
            r = self.rects[i]
            farbe = COL_SEL if i == self.sel else COL_BTN
            pygame.draw.rect(s, farbe, r, border_radius=8)
            img = self.font.render(label, True, COL_TEXT)
            s.blit(img, img.get_rect(center=r.center))

        hint = self.font.render(t("pregame.hint"), True, COL_MUTE)
        s.blit(hint, hint.get_rect(center=(self.width // 2, self.height - 24)))


# ---------------------------------------------------------------------------
#  Options-Screen: Sound/Haptik + Tastenbelegung
# ---------------------------------------------------------------------------

class OptionsScreen(_Screen):
    name = "Optionen"

    def __init__(self, surface, width, height, app, on_close):
        self.on_close = on_close
        super().__init__(surface, width, height, app)
        self.name = t("options.name")
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
        s.fill(COL_BG)

        title = self.font.render(t("options.title"), True, COL_TEXT)
        s.blit(title, (self._left_x, 24))

        # Überschrift für den Grafik-/Leistungs-Block (rechte Spalte oben)
        s.blit(self.font.render(t("options.graphics"), True, COL_ACCENT),
               (self._right_x, 40))

        # Spaltenüberschriften der Steuerung
        s.blit(self.font.render(t("common.player1"), True, COL_ACCENT),
               (self._left_x, 232))
        s.blit(self.font.render(t("common.player2"), True, COL_ACCENT),
               (self._right_x, 232))

        for i, it in enumerate(self.items):
            r = it["rect"]
            if i == self.sel:
                pygame.draw.rect(s, COL_PANEL, r.inflate(12, 6), border_radius=6)
            self._draw_item(it, selected=(i == self.sel))

        if self.capture is not None:
            self._draw_capture_overlay()

    def _draw_arrow_value(self, it, farbe, value_text, value_col):
        """Zeichnet 'Label            < Wert >' mit einzeln anklickbaren Pfeilen.

        Speichert die Trefferflächen der Pfeile in it['dec_rect']/it['inc_rect'].
        """
        s = self.surface
        r = it["rect"]
        s.blit(self.font.render(it["label"], True, farbe), (r.x, r.y))

        lt = self.font.render("<", True, farbe)
        val = self.font.render(str(value_text), True, value_col)
        gt = self.font.render(">", True, farbe)
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
            bar = pygame.Rect(r.x + 150, r.y + 8, 110, 12)
            it["bar_rect"] = bar
            pygame.draw.rect(s, COL_BTN, bar, border_radius=6)
            pygame.draw.rect(s, COL_ACCENT,
                             (bar.x, bar.y, int(bar.w * v), bar.h), border_radius=6)
            s.blit(self.font.render(f"{int(v * 100)}%", True, farbe),
                   (bar.right + 8, r.y))

        elif kind == "preset":
            self._draw_arrow_value(it, farbe,
                                   t(settings_mod.PRESETS[self.preset_idx][0]), COL_KEY)

        elif kind == "resolution":
            if self.settings.get("auto_resolution"):
                # Auto: keine Pfeile, aktuelle Fenster-Auflösung anzeigen.
                it["dec_rect"] = it["inc_rect"] = None
                s.blit(self.font.render(it["label"], True, farbe), (r.x, r.y))
                txt = t("options.res_auto", w=self.width, h=self.height)
                img = self.font.render(txt, True, COL_MUTE)
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
            s.blit(img, (r.right - img.get_width(), r.y))

        elif kind == "button":
            pygame.draw.rect(s, COL_SEL if selected else COL_BTN, r, border_radius=8)
            img = self.font.render(it["label"], True, COL_TEXT)
            s.blit(img, img.get_rect(center=r.center))

    def _draw_capture_overlay(self):
        s = self.surface
        overlay = pygame.Surface((self.width, self.height), pygame.SRCALPHA)
        overlay.fill((0, 0, 0, 170))
        s.blit(overlay, (0, 0))
        player, action = self.capture
        who = t("common.player1") if player == "p1" else t("common.player2")
        self.draw_center_text(f"{who}  -  {action_label(action)}",
                              self.font, COL_ACCENT, -40)
        self.draw_center_text(t("options.press_key"), self.big_font, COL_TEXT, 0)
        self.draw_center_text(t("options.cancel"), self.font, COL_MUTE, 44)


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
        s.fill(COL_BG)
        title = self.big_font.render(t("lang.title"), True, COL_TEXT)
        s.blit(title, title.get_rect(center=(self.width // 2, 90)))
        for i, (code, label) in enumerate(i18n.AVAILABLE):
            r = self.rects[i]
            pygame.draw.rect(s, COL_SEL if i == self.sel else COL_BTN, r, border_radius=10)
            img = self.big_font.render(label, True, COL_TEXT)
            s.blit(img, img.get_rect(center=r.center))
        hint = self.font.render(t("lang.hint"), True, COL_MUTE)
        s.blit(hint, hint.get_rect(center=(self.width // 2, self.height - 30)))
