# -*- coding: utf-8 -*-
"""
replayview.py
=============
Der Replay-Screen: Archiv und Wiedergabe der gespeicherten Wiederholungen.

Zwei Ansichten in einem Screen:

- Liste   : Reiter je Spiel (Minigolf/Bowling) mit allen gespeicherten
            Replays - Titel, Ergebnis, Datum und Laufzeit. Enter spielt ab,
            Entf löscht (zweimal drücken), Tab wechselt das Spiel.
- Player  : die eigentliche Wiedergabe. Gezeichnet wird sie vom Spiel selbst:
            der Screen baut eine ganz normale Spielinstanz und fährt sie über
            ``replay_begin``/``replay_seek``/``replay_draw`` Bild für Bild
            durch die Aufnahme (siehe replay.py). Damit sieht eine
            Wiederholung exakt aus wie die gespielte Runde - inklusive HUD
            und Scorekarte.

Jede Sequenz (Minigolf: ein Schlag, Bowling: ein Wurf) bekommt einen kurzen
Vorlauf mit Ziellinie und einen Nachlauf mit dem Ergebnis - so wirkt die
Wiederholung wie eine Zusammenfassung und nicht wie ein abgehacktes Video.

Der Screen wird an zwei Stellen geöffnet: über den Replays-Knopf in der
Sidebar (Archiv) und direkt aus Minigolf/Bowling am Rundenende über die
Taste P (dann liegt die frische, noch nicht gespeicherte Aufnahme bereit
und kann mit S ins Archiv gelegt werden).
"""

import pygame

import replay
import ui
from game_base import InputEvent
from i18n import t
from menu import _Screen

# Klassennamen der Spiele mit Aufzeichnung (Reihenfolge = Reiterleiste).
_CLASSES = {"minigolf": "MiniGolfGame", "bowling": "BowlingGame"}

# Zusatzbilder je Sequenz (in Samples, also 1/replay.RATE Sekunden):
# Vorlauf mit Ziellinie, kurzer Nachlauf, langer Nachlauf am Bahn-/Wurfende.
PRE = 12
POST = 9
POST_END = 27

# Wählbare Wiedergabe-Geschwindigkeiten.
SPEEDS = (0.5, 1.0, 2.0, 4.0)

# Zeilenhöhe in der Archiv-Liste und Höhe der Bedienleiste im Player.
ROW_H = 52
BAR_H = 46

# So lange bleibt die Bedienleiste nach der letzten Eingabe sichtbar.
BAR_SHOW = 3.0


def _game_class(key):
    """Spielklasse zu einem Replay-Schlüssel (oder None)."""
    import games
    return getattr(games, _CLASSES.get(key, ""), None)


def game_name(key):
    """Anzeigename des Spiels (sprachabhängig über die Spielklasse)."""
    cls = _game_class(key)
    return cls.name if cls is not None else key


def game_accent(key):
    """Akzentfarbe des Spiels (wie in der Sidebar-Liste)."""
    return ui.game_color(_CLASSES.get(key, ""), ui.ACCENT)


class ReplayScreen(_Screen):
    """Archiv + Wiedergabe. ``pending`` ist eine noch ungespeicherte Aufnahme."""

    name = "Replays"

    def __init__(self, surface, width, height, app, on_close,
                 pending=None, game=None):
        self.on_close = on_close
        self.pending = pending
        self.saved = False              # pending bereits ins Archiv gelegt?
        self.tab = game or (pending or {}).get("game") or replay.GAMES[0]
        if self.tab not in replay.GAMES:
            self.tab = replay.GAMES[0]
        self.mode = "list"
        self.sel = 0
        self.first = 0                  # erste sichtbare Zeile
        self._tab_hover = None
        self._hover_row = None
        self._confirm = None            # id des Replays, das gelöscht werden soll
        self.toast = None
        self.toast_t = 0.0
        self.items = []
        self.counts = {}

        # Wiedergabe-Zustand
        self.rep = None
        self.inst = None
        self.scenes = []
        self.lens = []                  # Bilder je Sequenz (inkl. Vor-/Nachlauf)
        self.starts = []                # Bild-Nummer, bei der eine Sequenz beginnt
        self.total = 0
        self.si = 0                     # aktuelle Sequenz
        self.pi = 0                     # Bild innerhalb der Sequenz
        self.playing = True
        self.speed_idx = 1
        self._acc = 0.0
        self._sound_done = False
        self.bar_t = BAR_SHOW
        self.error = None

        super().__init__(surface, width, height, app)
        self.name = t("replay.name")
        self._load()
        self._build()
        if pending is not None:
            self._start(pending)

    # ----- Daten & Layout -----------------------------------------------

    def _load(self):
        """Liest das Archiv neu ein (alle Spiele auf einmal, eine Datei)."""
        alle = replay.load_all()
        self.counts = {g: len(v) for g, v in alle.items()}
        self.items = alle.get(self.tab, [])
        self.sel = min(self.sel, max(0, len(self.items) - 1))
        self.first = 0
        self._confirm = None

    def on_surface_changed(self):
        self._build()
        if self.inst is not None:
            self.inst.surface = self.surface
            self.inst.width = self.width
            self.inst.height = self.height
            self.inst.on_surface_changed()
            self.inst._rep_at = None        # Pin-/Ballstand neu aufbauen
            self._seek_now()

    def _build(self):
        """Berechnet die Rechtecke von Reiterleiste, Liste und Leiste."""
        W, H = self.width, self.height
        self._left = max(24, W // 2 - 320)
        self._right = W - self._left
        self._top = 112
        self._bottom = H - 44

        self._tab_font = ui.font(14, bold=True)
        self.tab_rects = []
        tx = self._left
        for key in replay.GAMES:
            label = "%s (%d)" % (game_name(key), self.counts.get(key, 0))
            tw = self._tab_font.size(label)[0] + 26
            self.tab_rects.append((pygame.Rect(tx, 74, tw, 26), key, label))
            tx += tw + 8

        self.rows_visible = max(1, (self._bottom - self._top) // ROW_H)
        self.row_rects = [pygame.Rect(self._left, self._top + i * ROW_H,
                                      self._right - self._left, ROW_H - 6)
                          for i in range(self.rows_visible)]

        # Player: Bedienleiste unten + Fortschrittsbalken an ihrer Oberkante.
        self.bar_rect = pygame.Rect(0, H - BAR_H, W, BAR_H)
        self.seek_rect = pygame.Rect(0, H - BAR_H - 4, W, 10)
        chip_w = min(240, W - 40)
        self.save_rect = pygame.Rect(W // 2 - chip_w // 2, 10, chip_w, 28)
        self._chip_h = 28

    # ----- Wiedergabe starten/beenden ------------------------------------

    def _start(self, rep):
        """Baut die Spielinstanz auf und beginnt die Wiedergabe."""
        self.rep = rep
        self.error = None
        self.scenes = rep.get("scenes") or []
        cls = _game_class(rep.get("game"))
        self.inst = None
        if cls is not None and self.scenes:
            try:
                inst = cls(self.surface, self.width, self.height,
                           mode="single", game_settings=self.settings)
                inst.replay_begin(rep)
                self.inst = inst
            except Exception:           # defekte Aufnahme -> Hinweis statt Absturz
                self.inst = None
        if self.inst is None:
            self.error = t("replay.broken")
            self.mode = "list"
            return
        self.lens = []
        self.starts = []
        pos = 0
        for sc in self.scenes:
            n = max(1, replay.scene_len(sc))
            end = bool(sc.get("final") or sc.get("knocked") is not None)
            self.starts.append(pos)
            length = PRE + n + (POST_END if end else POST)
            self.lens.append(length)
            pos += length
        self.total = pos
        self.si = 0
        self.pi = 0
        self._acc = 0.0
        self._sound_done = False
        self.playing = True
        self.bar_t = BAR_SHOW
        self.mode = "play"
        self._seek_now()

    def _stop(self):
        """Beendet die Wiedergabe und geht zurück in die Liste."""
        self.inst = None
        self.rep = None
        self.mode = "list"
        self._load()
        self._build()

    def _close(self):
        """Escape: zurück ins Spiel bzw. ins Menü."""
        self.play_sound("click")
        self.on_close()

    # ----- Zeitachse ------------------------------------------------------

    def _rate(self):
        return float((self.rep or {}).get("rate") or replay.RATE)

    def _phase(self):
        """(bild_in_szene, aiming, banner) für den aktuellen Stand."""
        sc = self.scenes[self.si]
        n = max(1, replay.scene_len(sc))
        if self.pi < PRE:
            return 0, True, False
        if self.pi < PRE + n:
            return self.pi - PRE, False, False
        return n - 1, False, bool(sc.get("final"))

    def _seek_now(self):
        """Überträgt den aktuellen Stand in die Spielinstanz."""
        if self.inst is None or not self.scenes:
            return
        frame, _, _ = self._phase()
        self.inst.replay_seek(self.si, frame)

    def _elapsed(self):
        """Abgespielte Bilder seit dem Anfang der Aufnahme."""
        if not self.starts:
            return 0
        return self.starts[self.si] + self.pi

    def _at_end(self):
        return self.si >= len(self.scenes) - 1 and self.pi >= self.lens[-1] - 1

    def _advance(self, steps=1):
        """Spielt 'steps' Bilder weiter (hält am Ende an)."""
        for _ in range(steps):
            if self._at_end():
                self.playing = False
                return
            self.pi += 1
            if self.pi >= self.lens[self.si]:
                self.si += 1
                self.pi = 0
                self._sound_done = False
            elif not self._sound_done:
                # Beim Übergang in den Nachlauf das Ergebnis hörbar machen.
                sc = self.scenes[self.si]
                n = max(1, replay.scene_len(sc))
                if self.pi >= PRE + n:
                    self._sound_done = True
                    self._result_sound(sc)
        self._seek_now()

    def _result_sound(self, sc):
        key = sc.get("result") or ""
        if sc.get("end") == "cup" or key in ("bowl.strike",):
            self.play_sound("win")
        elif key in ("bowl.spare",) or sc.get("final"):
            self.play_sound("point")
        elif sc.get("end") == "water":
            self.play_sound("hit")

    def _jump_scene(self, d):
        """Eine Sequenz vor/zurück (Links/Rechts)."""
        if not self.scenes:
            return
        if d < 0 and self.pi > PRE + 4:
            self.pi = 0                     # erst an den Anfang der Sequenz
        else:
            self.si = max(0, min(len(self.scenes) - 1, self.si + d))
            self.pi = 0
        self._sound_done = False
        self.bar_t = BAR_SHOW
        self._seek_now()
        self.play_sound("move")

    def _seek_ratio(self, ratio):
        """Springt an eine Stelle der Gesamt-Zeitachse (Klick in die Leiste)."""
        if not self.total:
            return
        target = max(0, min(self.total - 1, int(self.total * ratio)))
        for i in range(len(self.lens) - 1, -1, -1):
            if target >= self.starts[i]:
                self.si = i
                self.pi = min(self.lens[i] - 1, target - self.starts[i])
                break
        self._sound_done = True
        self.bar_t = BAR_SHOW
        self._seek_now()

    # ----- Speichern / Löschen -------------------------------------------

    def _save_pending(self):
        """Legt die frische Aufnahme ins Archiv (Taste S)."""
        if self.pending is None:
            return
        if self.saved or replay.is_saved(self.pending.get("game"),
                                         self.pending.get("id")):
            self.saved = True
            self._toast(t("replay.already"))
            return
        ok, why = replay.save_replay(self.pending)
        if ok:
            self.saved = True
            self._toast(t("replay.saved"))
            self.play_sound("level")
            ui.spawn_burst(self.width // 2, 24, ui.GREEN)
            self._unlock_achievements()
        elif why == "full":
            self._toast(t("replay.full", n=replay.MAX_PER_GAME))
            self.play_sound("hit")
        else:
            self._toast(t("replay.error"))
            self.play_sound("hit")

    def _unlock_achievements(self):
        """Erfolge rund ums Archiv prüfen."""
        import achievements
        total = sum(len(v) for v in replay.load_all().values())
        achievements.event("replay_first")
        achievements.event("replay_5", total)

    def _delete_selected(self):
        """Entf: löscht das gewählte Replay (zweimal drücken)."""
        if not self.items or self.mode != "list":
            return
        rep = self.items[self.sel]
        if self._confirm != rep.get("id"):
            self._confirm = rep.get("id")
            self._toast(t("replay.confirm"))
            self.play_sound("move")
            return
        replay.delete_replay(self.tab, rep.get("id"))
        self._toast(t("replay.deleted"))
        self.play_sound("click")
        self._load()
        self._build()

    def _toast(self, text):
        self.toast = text
        self.toast_t = 2.6

    # ----- Eingabe --------------------------------------------------------

    def handle_event(self, event):
        if self.mode == "play":
            self._event_play(event)
        else:
            self._event_list(event)

    def _event_play(self, event):
        if event.kind == InputEvent.KEYDOWN:
            self.bar_t = BAR_SHOW
            k = event.key
            if k in ("Escape", "BackSpace"):
                if self.pending is not None:
                    self._close()
                else:
                    self._stop()
            elif k in ("space", "Return") or self.is_action(k, "action"):
                if self._at_end():
                    self.si = self.pi = 0
                    self._sound_done = False
                    self._seek_now()
                self.playing = not self.playing
                self.play_sound("click")
            elif k == "Left" or self.is_action(k, "left"):
                self._jump_scene(-1)
            elif k == "Right" or self.is_action(k, "right"):
                self._jump_scene(+1)
            elif k in ("Up", "plus", "KP_Add") or self.is_action(k, "up"):
                self._set_speed(+1)
            elif k in ("Down", "minus", "KP_Subtract") or self.is_action(k, "down"):
                self._set_speed(-1)
            elif k in ("s", "S"):
                self._save_pending()
        elif event.kind == InputEvent.MOUSEMOVE:
            self.bar_t = BAR_SHOW
        elif event.kind == InputEvent.MOUSEDOWN and event.button == 1:
            self.bar_t = BAR_SHOW
            if self.pending is not None and not self.saved \
                    and self.save_rect.collidepoint(event.pos):
                self._save_pending()
            elif self.seek_rect.collidepoint(event.pos) or \
                    self.bar_rect.collidepoint(event.pos):
                if self.seek_rect.collidepoint(event.pos):
                    self._seek_ratio(event.pos[0] / float(max(1, self.width)))
            else:
                self.playing = not self.playing
        elif event.kind == InputEvent.WHEEL:
            self._set_speed(1 if event.delta > 0 else -1)

    def _set_speed(self, d):
        idx = max(0, min(len(SPEEDS) - 1, self.speed_idx + d))
        if idx != self.speed_idx:
            self.speed_idx = idx
            self.play_sound("move")
        self.bar_t = BAR_SHOW

    def _event_list(self, event):
        if event.kind == InputEvent.KEYDOWN:
            k = event.key
            if k == "Escape":
                self._close()
            elif k == "Tab":
                self._switch_tab(replay.GAMES[(replay.GAMES.index(self.tab) + 1)
                                              % len(replay.GAMES)])
            elif k in ("Up", "w") or self.is_action(k, "up"):
                self._move(-1)
            elif k in ("Down", "s") or self.is_action(k, "down"):
                self._move(+1)
            elif k in ("Left", "a"):
                self._switch_tab(replay.GAMES[(replay.GAMES.index(self.tab) - 1)
                                              % len(replay.GAMES)])
            elif k in ("Right", "d"):
                self._switch_tab(replay.GAMES[(replay.GAMES.index(self.tab) + 1)
                                              % len(replay.GAMES)])
            elif k in ("Return", "space"):
                if self.items:
                    self.play_sound("click")
                    self._start(self.items[self.sel])
            elif k in ("Delete", "KP_Delete", "x", "X"):
                self._delete_selected()
        elif event.kind == InputEvent.MOUSEMOVE:
            self._tab_hover = None
            for r, key, _ in self.tab_rects:
                if r.collidepoint(event.pos):
                    self._tab_hover = key
            self._hover_row = None
            for i, r in enumerate(self.row_rects):
                if r.collidepoint(event.pos) and self.first + i < len(self.items):
                    self._hover_row = i
                    self.sel = self.first + i
        elif event.kind == InputEvent.MOUSEDOWN and event.button == 1:
            for r, key, _ in self.tab_rects:
                if r.collidepoint(event.pos):
                    self._switch_tab(key)
                    return
            for i, r in enumerate(self.row_rects):
                if r.collidepoint(event.pos) and self.first + i < len(self.items):
                    self.sel = self.first + i
                    self.play_sound("click")
                    self._start(self.items[self.sel])
                    return
        elif event.kind == InputEvent.WHEEL:
            self._move(-1 if event.delta > 0 else 1)

    def _move(self, d):
        if not self.items:
            return
        self.sel = (self.sel + d) % len(self.items)
        self._confirm = None
        if self.sel < self.first:
            self.first = self.sel
        elif self.sel >= self.first + self.rows_visible:
            self.first = self.sel - self.rows_visible + 1
        self.play_sound("move")

    def _switch_tab(self, key):
        if key == self.tab:
            return
        self.tab = key
        self.sel = 0
        self._load()
        self._build()
        self.play_sound("move")

    # ----- Ablauf ---------------------------------------------------------

    def update(self, dt):
        if self.toast_t > 0:
            self.toast_t -= dt
            if self.toast_t <= 0:
                self.toast = None
        if self.mode != "play" or self.inst is None:
            return
        if self.bar_t > 0:
            self.bar_t -= dt
        if not self.playing:
            return
        self._acc += dt * SPEEDS[self.speed_idx]
        step = 1.0 / self._rate()
        steps = 0
        while self._acc >= step and steps < 240:
            self._acc -= step
            steps += 1
        if steps:
            self._advance(steps)

    # ----- Zeichnen -------------------------------------------------------

    def draw(self):
        if self.mode == "play" and self.inst is not None:
            self._draw_play()
        else:
            self._draw_list()
        self._draw_toast()

    # --- Wiedergabe ---

    def _draw_play(self):
        _, aiming, banner = self._phase()
        self.inst.replay_draw(aiming=aiming, banner=banner)
        self._draw_chip()
        # Der Speichern-Hinweis bleibt, bis gespeichert wurde (danach kurz
        # noch als Bestaetigung, solange die Rueckmeldung sichtbar ist).
        if self.pending is not None and (not self.saved or self.toast_t > 0):
            self._draw_save_chip()
        if self.bar_t > 0 or not self.playing:
            self._draw_bar()

    def _chip_y(self):
        """Obere Kante der Chips: knapp unter HUD und Scorecard des Spiels."""
        inst = self.inst
        return getattr(inst, "hud_h", 40) + getattr(inst, "card_h", 0) + 8

    def _draw_chip(self):
        """Kleiner REPLAY-Chip mit pulsierendem Aufnahmepunkt."""
        s = self.surface
        fnt = ui.font(12, bold=True)
        img = fnt.render(t("replay.chip"), True, ui.TEXT)
        w = img.get_width() + 34
        h = img.get_height() + 10
        r = pygame.Rect(10, self._chip_y(), w, h)
        ui.draw_panel(s, r, radius=h // 2, shadow=False)
        col = ui.mix(ui.PANEL, ui.RED, 0.4 + 0.6 * ui.pulse(2.4))
        pygame.draw.circle(s, col, (r.x + 13, r.centery), 5)
        s.blit(img, img.get_rect(midleft=(r.x + 24, r.centery)))

    def _draw_save_chip(self):
        """Hinweis-Knopf zum Speichern der frischen Aufnahme (oben mittig)."""
        s = self.surface
        r = self.save_rect
        r.y = self._chip_y() - 1
        done = self.saved
        label = t("replay.saved_chip") if done else t("replay.save_chip")
        ui.draw_panel(s, r, radius=14, shadow=False,
                      color=ui.mix(ui.PANEL, ui.GREEN, 0.35 if done else 0.0))
        pygame.draw.rect(s, ui.GREEN if done else ui.ACCENT, r, 1,
                         border_radius=14)
        img = ui.font(13, bold=True).render(label, True,
                                            ui.GREEN if done else ui.TEXT)
        s.blit(img, img.get_rect(center=r.center))

    def _draw_bar(self):
        """Bedienleiste unten: Fortschritt, Titel, Sequenz, Tempo, Tasten."""
        s = self.surface
        W = self.width
        bar = self.bar_rect
        panel = pygame.Surface((bar.w, bar.h), pygame.SRCALPHA)
        panel.fill((10, 12, 18, 208))
        s.blit(panel, bar.topleft)
        accent = game_accent(self.rep.get("game"))
        pygame.draw.line(s, ui.BORDER, (0, bar.y), (W, bar.y))

        # Fortschritt über die volle Breite (Sequenz-Grenzen als Marken).
        y = bar.y - 3
        pygame.draw.rect(s, ui.PANEL, (0, y, W, 5))
        done = self._elapsed() / float(max(1, self.total))
        pygame.draw.rect(s, accent, (0, y, int(W * done), 5))
        for start in self.starts[1:]:
            x = int(W * start / float(max(1, self.total)))
            pygame.draw.line(s, ui.BORDER_LIGHT, (x, y), (x, y + 5))

        small = ui.font(13, bold=True)
        tiny = ui.font(11)
        # Links: Titel und Untertitel der Aufnahme.
        title = small.render(self.rep.get("title") or game_name(self.tab),
                             True, ui.TEXT)
        s.blit(title, (12, bar.y + 7))
        sub = tiny.render(self.rep.get("sub") or "", True, ui.TEXT_DIM)
        s.blit(sub, (12, bar.y + 26))

        # Mitte: Sequenz-Zähler + Zeit.
        rate = self._rate()
        pos_txt = "%s / %s" % (replay.format_duration(self._elapsed() / rate),
                               replay.format_duration(self.total / rate))
        mid = small.render(pos_txt, True, ui.TEXT_DIM)
        s.blit(mid, mid.get_rect(midtop=(W // 2, bar.y + 7)))
        label = t("replay.seq_golf" if self.rep.get("game") == "minigolf"
                  else "replay.seq_bowl",
                  n=self.si + 1, total=len(self.scenes))
        seq = tiny.render(label, True, accent)
        s.blit(seq, seq.get_rect(midtop=(W // 2, bar.y + 26)))

        # Rechts: Zustand und Tempo.
        state = t("replay.playing") if self.playing else t("replay.paused")
        st = small.render("%s  %.1fx" % (state, SPEEDS[self.speed_idx]),
                          True, ui.GOLD if not self.playing else ui.TEXT_DIM)
        s.blit(st, st.get_rect(topright=(W - 12, bar.y + 7)))
        hint = tiny.render(t("replay.hint_play"), True, ui.TEXT_FAINT)
        if hint.get_width() < W - 24:
            s.blit(hint, hint.get_rect(topright=(W - 12, bar.y + 26)))

    # --- Archiv-Liste ---

    def _draw_list(self):
        s = self.surface
        W, H = self.width, self.height
        ui.draw_background(s, W, H, stars=False)
        ui.draw_title(s, W, t("replay.name"), subtitle=t("replay.subtitle"),
                      y=26, big=ui.font(30, bold=True), accent=ui.ACCENT2)

        for r, key, label in self.tab_rects:
            active = (key == self.tab)
            hover = (key == self._tab_hover)
            col = ui.PANEL_LIGHT if (active or hover) else ui.PANEL
            pygame.draw.rect(s, col, r, border_radius=13)
            pygame.draw.rect(s, game_accent(key) if active else ui.BORDER, r, 1,
                             border_radius=13)
            img = self._tab_font.render(label, True,
                                        ui.TEXT if active else ui.TEXT_DIM)
            s.blit(img, img.get_rect(center=r.center))

        # Zähler rechts oben.
        cnt = ui.font(13, bold=True).render(
            t("replay.count", n=len(self.items), max=replay.MAX_PER_GAME),
            True, ui.TEXT_DIM)
        s.blit(cnt, cnt.get_rect(midright=(self._right, 87)))

        if not self.items:
            self._draw_empty()
        else:
            for i, r in enumerate(self.row_rects):
                idx = self.first + i
                if idx >= len(self.items):
                    break
                self._draw_row(s, r, self.items[idx], idx == self.sel)
            if len(self.items) > self.rows_visible:
                self._draw_scrollbar(s)

        ui.draw_footer(s, W, H, t("replay.hint_list"))

    def _draw_row(self, s, rect, rep, selected):
        accent = game_accent(rep.get("game"))
        col = ui.PANEL_LIGHT if selected else ui.PANEL
        ui.draw_panel(s, rect, color=col, radius=10, shadow=False)
        if selected:
            pygame.draw.rect(s, accent, rect, 1, border_radius=10)
        pygame.draw.rect(s, accent, (rect.x, rect.y + 8, 3, rect.h - 16),
                         border_radius=2)

        # Abspiel-Dreieck in der Spielfarbe.
        cx, cy = rect.x + 30, rect.centery
        pygame.draw.circle(s, ui.mix(ui.BTN, accent, 0.35), (cx, cy), 13)
        pygame.draw.polygon(s, ui.TEXT if selected else ui.TEXT_DIM,
                            [(cx - 4, cy - 6), (cx + 7, cy), (cx - 4, cy + 6)])

        title = ui.font(16, bold=True).render(rep.get("title") or "", True,
                                              ui.TEXT if selected else ui.TEXT_DIM)
        s.blit(title, (rect.x + 52, rect.y + 8))
        sub = ui.font(12).render(rep.get("sub") or "", True, ui.TEXT_FAINT)
        s.blit(sub, (rect.x + 52, rect.y + 28))

        dur = ui.font(13, bold=True).render(
            replay.format_duration(replay.duration(rep)), True, accent)
        s.blit(dur, dur.get_rect(topright=(rect.right - 14, rect.y + 8)))
        date = ui.font(11).render(rep.get("date") or "", True, ui.TEXT_FAINT)
        s.blit(date, date.get_rect(topright=(rect.right - 14, rect.y + 28)))

        if self._confirm == rep.get("id"):
            warn = ui.font(11, bold=True).render(t("replay.confirm_short"),
                                                 True, ui.RED)
            s.blit(warn, warn.get_rect(midright=(rect.right - 70,
                                                 rect.centery)))

    def _draw_empty(self):
        s = self.surface
        cx = self.width // 2
        cy = (self._top + self._bottom) // 2
        head = ui.font(17, bold=True).render(
            t("replay.empty", game=game_name(self.tab)), True, ui.TEXT_DIM)
        s.blit(head, head.get_rect(center=(cx, cy - 18)))
        hint = ui.font(13).render(t("replay.empty_hint"), True, ui.TEXT_FAINT)
        s.blit(hint, hint.get_rect(center=(cx, cy + 10)))
        if not replay.is_enabled(self.settings):
            off = ui.font(13, bold=True).render(t("replay.disabled"), True,
                                                ui.GOLD)
            s.blit(off, off.get_rect(center=(cx, cy + 38)))
        if self.error:
            err = ui.font(13, bold=True).render(self.error, True, ui.RED)
            s.blit(err, err.get_rect(center=(cx, cy + 62)))

    def _draw_scrollbar(self, s):
        top, bottom = self._top, self._top + self.rows_visible * ROW_H
        track = pygame.Rect(self._right + 6, top, 4, bottom - top)
        pygame.draw.rect(s, ui.PANEL, track, border_radius=2)
        frac = self.rows_visible / float(len(self.items))
        h = max(24, int(track.h * frac))
        pos = self.first / float(max(1, len(self.items) - self.rows_visible))
        y = top + int((track.h - h) * min(1.0, pos))
        pygame.draw.rect(s, ui.BORDER_LIGHT, (track.x, y, 4, h), border_radius=2)

    def _draw_toast(self):
        """Kurze Rückmeldung (gespeichert/gelöscht/voll) unten mittig."""
        if not self.toast:
            return
        s = self.surface
        fnt = ui.font(14, bold=True)
        img = fnt.render(self.toast, True, ui.TEXT)
        w, h = img.get_width() + 30, img.get_height() + 14
        y = self.height - h - (BAR_H + 14 if self.mode == "play" else 46)
        r = pygame.Rect(self.width // 2 - w // 2, y, w, h)
        ui.draw_panel(s, r, radius=h // 2, shadow=False)
        pygame.draw.rect(s, ui.ACCENT2, r, 1, border_radius=h // 2)
        s.blit(img, img.get_rect(center=r.center))
