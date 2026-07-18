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
        # Akzentfarbe des Spiels (gleiche Farbe wie in der Sidebar-Liste).
        self.accent = ui.game_color(game_cls.__name__)
        # Bisheriger Highscore für die Anzeige unter dem Titel.
        import highscore
        self.best = highscore.load_highscores().get(game_cls.highscore_key, 0)
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
        self.buttons.append((t("pregame.lamawiki"), self._open_lamawiki))
        self.buttons.append((t("pregame.back"), self.app.back_to_menu))

        # Rechtecke für Maus/Anzeige berechnen (zentriert, gestapelt).
        # Kleinere Buttons + Start unterhalb des Untertitels ("Modus wählen"),
        # damit dieser immer sichtbar bleibt (auch bei mehreren Modi).
        self.rects = []
        bw, bh, gap = 300, 40, 10
        total = len(self.buttons) * (bh + gap) - gap
        y0 = max(132, self.height // 2 - total // 2 + 6)
        # Viele Buttons (z.B. Solitär mit 5 Varianten) + kleine Auflösung:
        # kompakter stapeln, damit nichts unten herausläuft.
        if y0 + total > self.height - 40:
            bh, gap = 32, 6
            total = len(self.buttons) * (bh + gap) - gap
            y0 = max(120, self.height // 2 - total // 2 + 6)
        for i in range(len(self.buttons)):
            x = self.width // 2 - bw // 2
            y = y0 + i * (bh + gap)
            self.rects.append(pygame.Rect(x, y, bw, bh))

    def _open_options(self):
        opts = OptionsScreen(self.surface, self.width, self.height, self.app,
                             on_close=self._reopen)
        self.app.show_screen(opts)

    def _open_lamawiki(self):
        """Öffnet das LamaWiki direkt auf der Seite dieses Spiels."""
        from lamawiki import LamaWikiScreen, page_id_for_game
        self.app.show_screen(LamaWikiScreen(
            self.surface, self.width, self.height, self.app,
            on_close=self._reopen,
            page_id=page_id_for_game(self.game_cls.__name__)))

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
        # Kleiner Funken-Effekt am Button (wird zentral in main.py gezeichnet).
        r = self.rects[i]
        ui.spawn_burst(r.centerx, r.centery, self.accent)
        self.buttons[i][1]()

    def draw(self):
        s = self.surface
        ui.draw_background(s, self.width, self.height)

        ui.draw_title(s, self.width, self.game_cls.name,
                      subtitle=t("pregame.mode"), y=64, accent=self.accent)

        btn_font = ui.font(19)
        for i, (label, _) in enumerate(self.buttons):
            ui.draw_button(s, self.rects[i], label, btn_font,
                           selected=(i == self.sel), accent=self.accent)

        # Bisheriger Highscore als kleiner Chip über der Fußzeile
        # (nur wenn er nicht mit den Buttons kollidiert, z.B. bei 480p + vielen Modi).
        if self.best > 0:
            img = ui.font(14, bold=True).render(
                "★ " + t("app.highscore", hs=self.best), True, ui.GOLD)
            bw, bh = img.get_width() + 26, img.get_height() + 10
            chip = pygame.Rect(self.width // 2 - bw // 2,
                               self.height - 66 - bh // 2, bw, bh)
            if not self.rects or self.rects[-1].bottom + 8 < chip.top:
                ui.draw_panel(s, chip, radius=bh // 2, shadow=False)
                s.blit(img, img.get_rect(center=chip.center))

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

        # Karten-Panels hinter den Gruppen (rein optisch, mit Akzent-Lichtkante).
        for p in getattr(self, "_panels", ()):
            ui.draw_panel(s, p, radius=10, shadow=False,
                          accent_top=ui.mix(ui.PANEL, ui.ACCENT, 0.45))

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
#  Willkommens-Screen (nur beim allerersten Start)
# ---------------------------------------------------------------------------
#
#  Vereint die wichtigsten Ersteinstellungen auf EINER Seite, von oben nach
#  unten angeordnet - genau wie gewünscht:
#    - Auto-Auflösung an/aus   (ganz oben)
#    - feste Auflösung wählen   (mittig oben)
#    - Sound an/aus             (darunter; Standard: AUS)
#    - Sprache wählen           (unten: Englisch links, Deutsch mittig = Standard,
#                                Französisch rechts, "Mehr" für weitere Sprachen)
#  Bestätigt wird über den großen grünen "Los geht's"-Button: dabei werden die
#  Einstellungen gespeichert UND die Sprache dauerhaft in mem.json vermerkt,
#  damit dieser Screen beim nächsten Start nicht erneut erscheint.

class WelcomeScreen(_Screen):
    name = "Willkommen"

    # Anzeige-Reihenfolge der Hauptsprachen in der unteren Reihe:
    # Englisch links, Deutsch mittig (= Standard), Französisch rechts.
    PRIMARY_ORDER = ("en", "de", "fr")
    # Fokussierbare Reihen von oben nach unten (Auf/Ab bewegt sich hier durch).
    ROWS = ("auto", "resolution", "sound", "lang", "start")

    def __init__(self, surface, width, height, app, on_done):
        self.on_done = on_done
        self.expanded = False          # weitere Sprachen (es, pt) eingeblendet?
        super().__init__(surface, width, height, app)
        self.res_idx = settings_mod.resolution_index(
            self.settings.get("resolution", [640, 480]))
        self.name = t("welcome.name")
        self._build_langs()
        self._build()
        # Startfokus auf die Sprachreihe, Cursor auf der aktiven Sprache.
        self.row = self.ROWS.index("lang")
        self.lang_sel = self._active_lang_col()

    # ----- Aufbau / Layout ----------------------------------------------

    def _build_langs(self):
        """Legt die anzuzeigende Sprachreihe an (Hauptsprachen + evtl. weitere)."""
        names = dict(i18n.AVAILABLE)
        self.langs = [(c, names[c]) for c in self.PRIMARY_ORDER if c in names]
        if self.expanded:
            self.langs += [(c, n) for c, n in i18n.EXTRA]

    def _active_lang_col(self):
        """Index der aktuell aktiven Sprache in self.langs (sonst 0)."""
        cur = i18n.get_language()
        for i, (c, _) in enumerate(self.langs):
            if c == cur:
                return i
        return 0

    def _build(self):
        W, H = self.width, self.height
        cx = W // 2
        compact = H < 380
        self.panel_w = min(400, W - 40)
        px = cx - self.panel_w // 2
        self.ctrl_x = px + 18
        ctrl_w = self.panel_w - 36

        head1_y = 96 if compact else 112
        gap = 26 if compact else 34
        self.head1_y = head1_y
        self.rect_auto = pygame.Rect(self.ctrl_x, head1_y + 24, ctrl_w, 26)
        self.rect_res = pygame.Rect(self.ctrl_x, self.rect_auto.y + gap, ctrl_w, 26)
        self.rect_sound = pygame.Rect(self.ctrl_x, self.rect_res.y + gap, ctrl_w, 26)
        self.panel1 = pygame.Rect(px, head1_y - 6, self.panel_w,
                                  (self.rect_sound.bottom + 12) - (head1_y - 6))

        # Sprach-Überschrift nur bei genug Höhe (spart Platz auf 360px-Screens).
        self.show_lang_head = not compact
        if compact:
            self.lang_head_y = self.panel1.bottom + 12
            lang_row_y = self.panel1.bottom + 12
        else:
            self.lang_head_y = self.panel1.bottom + 14
            lang_row_y = self.lang_head_y + 24
        self._layout_langs(lang_row_y)

        bw = min(300, W - 100)
        start_y = self._lang_bottom + (12 if compact else 18)
        self.rect_start = pygame.Rect(cx - bw // 2, start_y, bw,
                                      34 if compact else 40)

    def _layout_langs(self, y):
        """Ordnet die Sprach-Buttons (eine Reihe) plus 'Mehr'-Button an."""
        W, H = self.width, self.height
        cx = W // 2
        gap = 8
        lh = 36 if H < 380 else 48
        area_w = min(452, W - 24)
        n = len(self.langs)
        if self.expanded:
            langs_w, other_w = area_w, 0
        else:
            other_w = max(60, min(84, area_w // 6))
            langs_w = area_w - other_w - gap
        bwid = max(1, (langs_w - (n - 1) * gap) // n)
        x0 = cx - area_w // 2
        self.lang_rects = [pygame.Rect(x0 + i * (bwid + gap), y, bwid, lh)
                           for i in range(n)]
        if self.expanded:
            self.other_rect = None
        else:
            self.other_rect = pygame.Rect(x0 + langs_w + gap, y, other_w, lh)
        self._lang_bwid = bwid
        self._lang_bottom = y + lh

    def on_surface_changed(self):
        """Nach einer Auflösungsänderung das Layout neu berechnen."""
        self._build_langs()
        self._build()
        self.row = min(self.row, len(self.ROWS) - 1)
        self.lang_sel = min(self.lang_sel, len(self._lang_targets()) - 1)

    def _lang_targets(self):
        """Fokussierbare Ziele der Sprachreihe: alle Sprachen + evtl. 'Mehr'."""
        ts = [("lang", i) for i in range(len(self.langs))]
        if self.other_rect is not None:
            ts.append(("other", None))
        return ts

    # ----- Eingabe ------------------------------------------------------

    def handle_event(self, event):
        if event.kind == InputEvent.KEYDOWN:
            self._on_key(event.key)
        elif event.kind == InputEvent.MOUSEMOVE:
            self._hover(event.pos)
        elif event.kind == InputEvent.MOUSEDOWN:
            self._hover(event.pos)
            self._click(event.pos)

    def _on_key(self, key):
        if key == "Escape":
            self._finish()                 # Erststart: Escape übernimmt & startet
        elif key in ("Up", "w"):
            self.row = (self.row - 1) % len(self.ROWS)
            self.play_sound("move")
        elif key in ("Down", "s"):
            self.row = (self.row + 1) % len(self.ROWS)
            self.play_sound("move")
        elif key in ("Left", "a"):
            self._horizontal(-1)
        elif key in ("Right", "d"):
            self._horizontal(+1)
        elif key in ("Return", "space"):
            self._activate()

    def _horizontal(self, d):
        kind = self.ROWS[self.row]
        if kind == "auto":
            self._set_auto(d > 0)
        elif kind == "sound":
            self._set_sound(d > 0)
        elif kind == "resolution":
            self._change_res(d)
        elif kind == "lang":
            self.lang_sel = (self.lang_sel + d) % len(self._lang_targets())
            self.play_sound("move")

    def _activate(self):
        kind = self.ROWS[self.row]
        if kind == "auto":
            self._set_auto(not self.settings.get("auto_resolution", False))
        elif kind == "sound":
            self._set_sound(not self.settings.get("sound", False))
        elif kind == "resolution":
            self._change_res(+1)
        elif kind == "lang":
            tkind, idx = self._lang_targets()[self.lang_sel]
            self._select_lang(idx) if tkind == "lang" else self._expand()
        elif kind == "start":
            self._finish()

    def _hover(self, pos):
        if self.rect_auto.collidepoint(pos):
            self.row = self.ROWS.index("auto")
        elif self.rect_res.collidepoint(pos):
            self.row = self.ROWS.index("resolution")
        elif self.rect_sound.collidepoint(pos):
            self.row = self.ROWS.index("sound")
        elif self.rect_start.collidepoint(pos):
            self.row = self.ROWS.index("start")
        else:
            for i, r in enumerate(self.lang_rects):
                if r and r.collidepoint(pos):
                    self.row = self.ROWS.index("lang")
                    self.lang_sel = i
                    return
            if self.other_rect and self.other_rect.collidepoint(pos):
                self.row = self.ROWS.index("lang")
                self.lang_sel = len(self.langs)   # "Mehr" ist das letzte Ziel

    def _click(self, pos):
        if self.rect_auto.collidepoint(pos):
            self._set_auto(not self.settings.get("auto_resolution", False))
        elif self.rect_sound.collidepoint(pos):
            self._set_sound(not self.settings.get("sound", False))
        elif self.rect_res.collidepoint(pos):
            dec, inc = getattr(self, "_res_dec", None), getattr(self, "_res_inc", None)
            if dec and dec.collidepoint(pos):
                self._change_res(-1)
            elif inc and inc.collidepoint(pos):
                self._change_res(+1)
            elif not self.settings.get("auto_resolution"):
                self._change_res(+1)
        elif self.rect_start.collidepoint(pos):
            self._finish()
        else:
            for i, r in enumerate(self.lang_rects):
                if r and r.collidepoint(pos):
                    self._select_lang(i)
                    return
            if self.other_rect and self.other_rect.collidepoint(pos):
                self._expand()

    # ----- Aktionen -----------------------------------------------------

    def _save(self):
        settings_mod.save_settings(self.settings)

    def _set_auto(self, on):
        on = bool(on)
        if self.settings.get("auto_resolution", False) == on:
            return
        self.settings["auto_resolution"] = on
        self.app.set_auto_resolution(on)   # wendet sofort an (Fenster/feste Auflösung)
        self._save()
        self.play_sound("select")

    def _set_sound(self, on):
        on = bool(on)
        if self.settings.get("sound", False) == on:
            return
        self.settings["sound"] = on
        self._save()
        self.play_sound("select")          # nur hörbar, wenn Sound nun an ist

    def _change_res(self, d):
        if self.settings.get("auto_resolution"):
            return
        self.res_idx = (self.res_idx + d) % len(settings_mod.RESOLUTIONS)
        w, h = settings_mod.RESOLUTIONS[self.res_idx][1]
        self.app.apply_resolution(w, h)    # setzt die Fläche neu -> on_surface_changed()
        self._save()
        self.play_sound("select")

    def _select_lang(self, i):
        i18n.set_language(self.langs[i][0])   # persistiert die Sprache in mem.json
        self.app.refresh_language()           # Tkinter-Menü neu beschriften
        self.name = t("welcome.name")
        self._build_langs()
        self._build()                         # Beschriftungen dieses Screens neu
        self.row = self.ROWS.index("lang")
        self.lang_sel = i
        self.play_sound("click")
        r = self.lang_rects[i]
        if r:
            ui.spawn_burst(r.centerx, r.centery, ui.ACCENT)

    def _expand(self):
        if self.expanded:
            return
        self.expanded = True
        self._build_langs()
        self._build()
        self.lang_sel = len(self.PRIMARY_ORDER)   # Fokus auf erste neue Sprache
        self.play_sound("click")

    def _finish(self):
        # Sprache dauerhaft vermerken (auch wenn der Standard beibehalten wurde),
        # damit der Willkommens-Screen künftig nicht erneut erscheint.
        i18n.set_language(i18n.get_language())
        self._save()
        self.play_sound("select")
        ui.spawn_burst(self.rect_start.centerx, self.rect_start.centery, ui.GREEN)
        self.on_done()

    # ----- Zeichnen -----------------------------------------------------

    def draw(self):
        s = self.surface
        compact = self.height < 380
        ui.draw_background(s, self.width, self.height)
        ui.draw_title(s, self.width, "PyGameZ",
                      subtitle=t("welcome.subtitle"),
                      y=30 if compact else 42,
                      big=ui.font(26 if compact else 32, bold=True),
                      small=ui.font(14 if compact else 16))

        # Karte "Bild & Ton": Auto-Auflösung / Auflösung / Sound.
        ui.draw_panel(s, self.panel1, radius=10, shadow=False,
                      accent_top=ui.mix(ui.PANEL, ui.ACCENT, 0.45))
        s.blit(ui.font(14, bold=True).render(t("welcome.setup"), True, ui.ACCENT),
               (self.ctrl_x, self.head1_y + 2))

        auto_on = self.settings.get("auto_resolution", False)
        self._draw_toggle(self.rect_auto, t("options.auto_res"), auto_on,
                          self.row == self.ROWS.index("auto"))
        self._draw_resolution(self.rect_res,
                              self.row == self.ROWS.index("resolution"), auto_on)
        self._draw_toggle(self.rect_sound, t("options.sound"),
                          self.settings.get("sound", False),
                          self.row == self.ROWS.index("sound"))

        # Sprach-Überschrift + Sprach-Buttons.
        if self.show_lang_head:
            head = ui.font(15, bold=True).render(t("lang.name"), True, ui.ACCENT)
            s.blit(head, head.get_rect(midtop=(self.width // 2, self.lang_head_y)))

        lang_focus = (self.row == self.ROWS.index("lang"))
        bw = self._lang_bwid
        bsize = 20 if bw >= 100 else (16 if bw >= 82 else 14)
        if compact:
            bsize = max(13, bsize - 2)
        bfont = ui.font(bsize, bold=True)
        cfont = ui.font(11)
        cur = i18n.get_language()
        for i, (code, label) in enumerate(self.langs):
            r = self.lang_rects[i]
            if not r:
                continue
            ui.draw_button(s, r, label, bfont,
                           selected=(lang_focus and self.lang_sel == i),
                           sub=code.upper(), sub_font=cfont)
            # Aktuell aktive Sprache dezent grün umranden (auch ohne Fokus).
            if code == cur:
                pygame.draw.rect(s, ui.GREEN, r.inflate(4, 4), width=2,
                                 border_radius=12)

        if self.other_rect is not None:
            hot = lang_focus and self.lang_sel == len(self.langs)
            ui.draw_button(s, self.other_rect, t("welcome.more"), ui.font(15),
                           selected=hot, accent=ui.ACCENT2)

        # Großer grüner Start-Button.
        ui.draw_button(s, self.rect_start, t("welcome.start"),
                       ui.font(19 if compact else 22, bold=True),
                       selected=(self.row == self.ROWS.index("start")),
                       accent=ui.GREEN)

        ui.draw_footer(s, self.width, self.height, t("welcome.hint"))

    def _draw_row_focus(self, rect):
        """Weiche Auswahl-Fläche + Akzentbalken links (wie im Options-Screen)."""
        s = self.surface
        hl = rect.inflate(16, 8)
        pygame.draw.rect(s, ui.PANEL_LIGHT, hl, border_radius=6)
        pygame.draw.rect(s, ui.ACCENT, (hl.x, hl.y + 3, 3, hl.h - 6), border_radius=2)

    def _draw_toggle(self, rect, label, on, focused):
        s = self.surface
        if focused:
            self._draw_row_focus(rect)
        col = COL_TEXT if focused else COL_MUTE
        img = ui.font(17).render(label, True, col)
        s.blit(img, (rect.x, rect.centery - img.get_height() // 2))
        # Schiebe-Pille rechts + Zustandswort AN/AUS links davon.
        pill_w, pill_h = 46, 22
        pill = pygame.Rect(rect.right - pill_w, rect.centery - pill_h // 2,
                           pill_w, pill_h)
        pygame.draw.rect(s, ui.mix(ui.BTN, ui.GREEN, 1.0 if on else 0.0), pill,
                         border_radius=pill_h // 2)
        pygame.draw.rect(s, ui.BORDER_LIGHT, pill, width=1, border_radius=pill_h // 2)
        knob_r = pill_h // 2 - 3
        knob_x = pill.right - knob_r - 4 if on else pill.left + knob_r + 4
        pygame.draw.circle(s, COL_TEXT, (knob_x, pill.centery), knob_r)
        word = ui.font(13, bold=True).render(t("common.on") if on else t("common.off"),
                                             True, COL_ACCENT if on else COL_MUTE)
        s.blit(word, (pill.left - 8 - word.get_width(),
                      rect.centery - word.get_height() // 2))

    def _draw_resolution(self, rect, focused, auto_on):
        s = self.surface
        if focused:
            self._draw_row_focus(rect)
        col = COL_TEXT if focused else COL_MUTE
        f = ui.font(17)
        label = f.render(t("options.resolution"), True, col)
        s.blit(label, (rect.x, rect.centery - label.get_height() // 2))
        if auto_on:
            # Auto-Modus: keine Pfeile, aktuelle Fenster-Auflösung anzeigen.
            self._res_dec = self._res_inc = None
            img = f.render(t("options.res_auto", w=self.width, h=self.height),
                           True, COL_MUTE)
            s.blit(img, (rect.right - img.get_width(),
                         rect.centery - img.get_height() // 2))
            return
        lt = f.render("<", True, col)
        gt = f.render(">", True, col)
        val = ui.font(15).render(t(settings_mod.RESOLUTIONS[self.res_idx][0]),
                                 True, COL_KEY)
        pad = 10
        gx = rect.right - gt.get_width()
        vx = gx - pad - val.get_width()
        lx = vx - pad - lt.get_width()
        cy = rect.centery
        s.blit(lt, (lx, cy - lt.get_height() // 2))
        s.blit(val, (vx, cy - val.get_height() // 2))
        s.blit(gt, (gx, cy - gt.get_height() // 2))
        self._res_dec = pygame.Rect(lx - 6, rect.y - 4, lt.get_width() + 12, rect.h + 8)
        self._res_inc = pygame.Rect(gx - 6, rect.y - 4, gt.get_width() + 12, rect.h + 8)


# ---------------------------------------------------------------------------
#  Sprachauswahl-Screen (beim ersten Start; zweisprachige Beschriftung)
# ---------------------------------------------------------------------------

class LanguageScreen(_Screen):
    name = "Sprache"

    def __init__(self, surface, width, height, app, on_done):
        self.on_done = on_done
        # Beim ersten Start (noch keine Sprache gewählt) werden nur die
        # Hauptsprachen angezeigt; die weiteren verstecken sich hinter einem
        # dezenten "Weitere Sprachen"-Button ganz unten. Wird der Screen später
        # aus dem Menü geöffnet, erscheinen sofort alle Sprachen.
        self.expanded = i18n.has_language()
        super().__init__(surface, width, height, app)
        self.name = t("lang.name")
        self._build()
        codes = [c for c, _ in self.langs]
        self.sel = codes.index(i18n.get_language()) if i18n.get_language() in codes else 0

    def _build(self):
        self.langs = list(i18n.AVAILABLE) if self.expanded else list(i18n.PRIMARY)
        self.rects = []
        n = len(self.langs)
        bw = min(300, self.width - 40)
        # Platz unter dem Titel; im eingeklappten Zustand bleibt unten Raum
        # für den "Weitere Sprachen"-Button.
        top = 124
        bottom = self.height - (44 if self.expanded else 88)
        bh, gap = 56, 16
        while n * (bh + gap) - gap > bottom - top and bh > 30:
            bh -= 2
            gap = max(6, gap - 1)
        total = n * (bh + gap) - gap
        y0 = top + max(0, (bottom - top - total) // 2)
        x = self.width // 2 - bw // 2
        for i in range(n):
            self.rects.append(pygame.Rect(x, y0 + i * (bh + gap), bw, bh))
        # Kleiner, bewusst unauffälliger Button ganz unten (nur eingeklappt).
        if self.expanded:
            self.other_rect = None
        else:
            f = ui.font(14)
            w = f.size(t("lang.other"))[0] + 26
            self.other_rect = pygame.Rect(self.width // 2 - w // 2,
                                          self.height - 66, w, 24)

    def _item_count(self):
        """Anzahl wählbarer Einträge (Sprachen + ggf. 'Weitere Sprachen')."""
        return len(self.rects) + (0 if self.expanded else 1)

    def on_surface_changed(self):
        self._build()
        self.sel = min(self.sel, self._item_count() - 1)

    def _expand(self):
        """Zeigt die zusätzlichen Sprachen (es, pt) an."""
        self.expanded = True
        self._build()
        # Auswahl auf die erste neu eingeblendete Sprache setzen.
        self.sel = min(len(i18n.PRIMARY), len(self.rects) - 1)
        self.play_sound("click")

    def _choose(self, i):
        if not self.expanded and i >= len(self.rects):
            self._expand()
            return
        i18n.set_language(self.langs[i][0])
        self.app.refresh_language()
        self.play_sound("click")
        r = self.rects[i]
        ui.spawn_burst(r.centerx, r.centery, ui.ACCENT)
        self.on_done()

    def handle_event(self, event):
        if event.kind == InputEvent.KEYDOWN:
            if event.key in ("Up", "w", "Left", "a"):
                self.sel = (self.sel - 1) % self._item_count()
                self.play_sound("move")
            elif event.key in ("Down", "s", "Right", "d"):
                self.sel = (self.sel + 1) % self._item_count()
                self.play_sound("move")
            elif event.key in ("Return", "space"):
                self._choose(self.sel)
        elif event.kind == InputEvent.MOUSEMOVE:
            for i, r in enumerate(self.rects):
                if r.collidepoint(event.pos):
                    self.sel = i
            if self.other_rect and self.other_rect.collidepoint(event.pos):
                self.sel = len(self.rects)
        elif event.kind == InputEvent.MOUSEDOWN:
            for i, r in enumerate(self.rects):
                if r.collidepoint(event.pos):
                    self._choose(i)
            if self.other_rect and self.other_rect.collidepoint(event.pos):
                self._choose(len(self.rects))

    def draw(self):
        s = self.surface
        ui.draw_background(s, self.width, self.height)
        ui.draw_title(s, self.width, t("lang.title"), y=80,
                      big=ui.font(34, bold=True))
        btn_font = ui.font(24, bold=True)
        code_font = ui.font(13)
        for i, (code, label) in enumerate(self.langs):
            ui.draw_button(s, self.rects[i], label, btn_font,
                           selected=(i == self.sel),
                           sub=code.upper(), sub_font=code_font)
        if self.other_rect:
            # Bewusst dezent: kleiner Text, gedeckte Farbe, feiner Rahmen
            # nur bei Auswahl - kein Akzent-Button wie die Sprachen darüber.
            hot = (self.sel == len(self.rects))
            col = COL_MUTE if hot else ui.TEXT_FAINT
            img = ui.font(14).render(t("lang.other"), True, col)
            if hot:
                pygame.draw.rect(s, ui.PANEL_LIGHT, self.other_rect,
                                 border_radius=6)
                pygame.draw.rect(s, ui.BORDER_LIGHT, self.other_rect,
                                 width=1, border_radius=6)
            s.blit(img, img.get_rect(center=self.other_rect.center))
        ui.draw_footer(s, self.width, self.height, t("lang.hint"))
