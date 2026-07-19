# -*- coding: utf-8 -*-
"""
pong.py
=======
Pong - Einzelspieler (gegen KI) oder Mehrspieler (2 Spieler).

- Steuerung über die belegten Tasten (Standard: Spieler 1 = W/S, Spieler 2 =
  Pfeil hoch/runter). Die Aktions-Taste hält den eigenen Schläger an.
- Einzelspieler: rechter Schläger wird von einer einfachen KI gesteuert.
- Mehrspieler: rechter Schläger = Spieler 2.
- Bewegungsmodus pro Steuerung umschaltbar (im Spiel):
    * Dauer  : einmal drücken -> der Schläger fährt dauerhaft weiter
               (bis Richtungswechsel oder Aktions-Taste). Das ist der Standard.
    * Halten : der Schläger bewegt sich nur, solange man die Taste gedrückt
               HÄLT (nutzt die Tastenwiederholung), und stoppt beim Loslassen.
  Umschalttasten:  X = Steuerung 1 (z.B. WASD),  N = Steuerung 2 (z.B. IJKL/Pfeile).
  Die Einstellung wird dauerhaft in settings.json ("pong") gespeichert.
- Ball-Physik mit Beschleunigung und Winkel je nach Treffpunkt.
- Es wird bis 5 Punkte gespielt. Als Highscore zählen die Punkte links (P1).
- Optik: Themen-Hintergrund und dynamische ui.*-Palette; Schriftgrössen und
  Positionen passen sich der Auflösung an (on_surface_changed).
"""

import random
import pygame

import settings as settings_mod
import ui
from game_base import Game, InputEvent
from i18n import t

# Identitätsfarben des Spielfelds (bewusst fest, unabhängig vom Theme).
COL_BALL = (255, 220, 90)    # klassischer gelber Ball
COL_P1 = (140, 230, 160)     # linker Schläger (Spieler 1)
COL_P2 = (150, 200, 255)     # rechter Schläger (Spieler 2)

PADDLE_W = 12
PADDLE_H = 80
PADDLE_SPEED = 380          # Pixel/Sekunde (Spieler)
AI_SPEED = 300              # Pixel/Sekunde (KI, absichtlich langsamer -> schlagbar)
BALL_SIZE = 14
BALL_START_SPEED = 320
BALL_MAX_SPEED = 650
WIN_SCORE = 5

# So lange (Sekunden) fährt der Schläger im "Halten"-Modus nach dem letzten
# Tastendruck noch weiter. Die Tastenwiederholung feuert schneller als das,
# daher bewegt er sich flüssig solange man hält - und stoppt kurz nach dem
# Loslassen (wenn keine Wiederholung mehr kommt).
HOLD_KEEPALIVE = 0.12

# Feste Umschalttasten für den Bewegungsmodus.
TOGGLE_KEYS = {"x": "p1", "X": "p1", "n": "p2", "N": "p2"}


class PongGame(Game):
    name = "Pong"
    highscore_key = "pong"
    supports_multiplayer = True

    def reset(self):
        self.score = 0            # = Punkte des linken Spielers (P1)
        self.game_over = False

        self.player_y = self.height / 2 - PADDLE_H / 2
        self.ai_y = self.height / 2 - PADDLE_H / 2
        self.player_score = 0
        self.ai_score = 0

        # Bewegungszustand pro Steuerung ("p1"/"p2"):
        #   dir  : aktuelle Richtung (-1/0/1)
        #   keep : Rest-Zeit im "Halten"-Modus, bis der Schläger stoppt
        self.dir = {"p1": 0, "p2": 0}
        self.keep = {"p1": 0.0, "p2": 0.0}
        pg = self.settings.get("pong", {}) if isinstance(self.settings, dict) else {}
        self.hold = {"p1": bool(pg.get("hold_p1", False)),
                     "p2": bool(pg.get("hold_p2", False))}

        self._make_fonts()

        self._serve(toward_player=random.choice([True, False]))

    # ----- Layout / Theme ------------------------------------------------

    def _make_fonts(self):
        """Themen-Schriften, Grössen aus der Fensterhöhe abgeleitet."""
        h = self.height
        self.font = ui.font(max(16, h // 26))
        self.big_font = ui.font(max(36, h // 10), bold=True)
        self._small = ui.font(max(13, h // 36))

    def on_surface_changed(self):
        """Auflösungswechsel: Schriften neu aufbauen und alle
        beweglichen Objekte in das neue Spielfeld einpassen."""
        self._make_fonts()
        self.player_y = max(0, min(self.height - PADDLE_H, self.player_y))
        self.ai_y = max(0, min(self.height - PADDLE_H, self.ai_y))
        self.ball_x = max(0.0, min(float(self.width - BALL_SIZE), self.ball_x))
        self.ball_y = max(0.0, min(float(self.height - BALL_SIZE), self.ball_y))

    def _serve(self, toward_player):
        """Setzt den Ball in die Mitte und gibt ihm eine Startrichtung."""
        self.ball_x = self.width / 2 - BALL_SIZE / 2
        self.ball_y = self.height / 2 - BALL_SIZE / 2
        winkel = random.uniform(-0.35, 0.35)
        richtung = -1 if toward_player else 1
        self.ball_vx = richtung * BALL_START_SPEED
        self.ball_vy = BALL_START_SPEED * winkel

    def handle_event(self, event):
        if event.kind != InputEvent.KEYDOWN:
            return

        if self.game_over:
            if event.key in ("Return", "space"):
                self.reset()
            return

        # Bewegungsmodus umschalten: X = Steuerung 1, N = Steuerung 2.
        if event.key in TOGGLE_KEYS:
            self._toggle_hold(TOGGLE_KEYS[event.key])
            return

        # Bewegung je Steuerung auswerten. Im Einzelspieler steuern BEIDE
        # Belegungen den linken Schläger, im Mehrspieler p1=links, p2=rechts.
        self._move_scheme("p1", event.key)
        self._move_scheme("p2", event.key)

    def _move_scheme(self, scheme, key):
        """Setzt Richtung/Keepalive für eine Steuerung anhand einer Taste."""
        if self.is_action(key, "up", scheme):
            self._press(scheme, -1)
        elif self.is_action(key, "down", scheme):
            self._press(scheme, 1)
        elif self.is_action(key, "action", scheme):
            # Aktions-Taste hält im Dauer-Modus an (im Halten-Modus unnötig).
            self.dir[scheme] = 0
            self.keep[scheme] = 0.0

    def _press(self, scheme, richtung):
        self.dir[scheme] = richtung
        if self.hold[scheme]:
            # Im Halten-Modus fährt der Schläger nur kurz weiter; die
            # Tastenwiederholung frischt das laufend auf, solange man hält.
            self.keep[scheme] = HOLD_KEEPALIVE

    def _toggle_hold(self, scheme):
        """Schaltet für eine Steuerung zwischen Dauer- und Halten-Modus um."""
        self.hold[scheme] = not self.hold[scheme]
        # Laufende Bewegung beim Umschalten stoppen.
        self.dir[scheme] = 0
        self.keep[scheme] = 0.0
        if isinstance(self.settings, dict):
            self.settings.setdefault("pong", {})[f"hold_{scheme}"] = self.hold[scheme]
            settings_mod.save_settings(self.settings)
        self.play_sound("select")

    def update(self, dt):
        if self.game_over:
            return

        # Halten-Modus: Bewegung ausklingen lassen, wenn keine Tastenwiederholung
        # mehr kommt (Taste losgelassen).
        for scheme in ("p1", "p2"):
            if self.hold[scheme] and self.dir[scheme] != 0:
                self.keep[scheme] -= dt
                if self.keep[scheme] <= 0:
                    self.dir[scheme] = 0

        # --- Linker Schläger ---
        if self.multiplayer:
            left_dir = self.dir["p1"]
        else:
            # Einzelspieler: beide Steuerungen bewegen den linken Schläger.
            left_dir = max(-1, min(1, self.dir["p1"] + self.dir["p2"]))
        self.player_y += left_dir * PADDLE_SPEED * dt
        self.player_y = max(0, min(self.height - PADDLE_H, self.player_y))

        # --- Rechter Schläger: KI (Einzel) oder Spieler 2 (Mehrspieler) ---
        if self.multiplayer:
            self.ai_y += self.dir["p2"] * PADDLE_SPEED * dt
        else:
            ziel = self.ball_y + BALL_SIZE / 2 - PADDLE_H / 2
            if self.ai_y < ziel - 8:
                self.ai_y += AI_SPEED * dt
            elif self.ai_y > ziel + 8:
                self.ai_y -= AI_SPEED * dt
        self.ai_y = max(0, min(self.height - PADDLE_H, self.ai_y))

        # --- Ball bewegen ---
        self.ball_x += self.ball_vx * dt
        self.ball_y += self.ball_vy * dt

        if self.ball_y <= 0:
            self.ball_y = 0
            self.ball_vy = abs(self.ball_vy)
            self.play_sound("bounce")
        elif self.ball_y + BALL_SIZE >= self.height:
            self.ball_y = self.height - BALL_SIZE
            self.ball_vy = -abs(self.ball_vy)
            self.play_sound("bounce")

        ball_rect = pygame.Rect(self.ball_x, self.ball_y, BALL_SIZE, BALL_SIZE)
        player_rect = pygame.Rect(20, self.player_y, PADDLE_W, PADDLE_H)
        ai_rect = pygame.Rect(self.width - 20 - PADDLE_W, self.ai_y, PADDLE_W, PADDLE_H)

        if ball_rect.colliderect(player_rect) and self.ball_vx < 0:
            self._bounce(player_rect, nach_rechts=True)
        elif ball_rect.colliderect(ai_rect) and self.ball_vx > 0:
            self._bounce(ai_rect, nach_rechts=False)

        # --- Punkte ---
        if self.ball_x + BALL_SIZE < 0:          # rechts punktet
            self.ai_score += 1
            self.play_sound("point")
            self._nach_punkt(toward_player=True)
        elif self.ball_x > self.width:           # links punktet
            self.player_score += 1
            self.score = self.player_score
            self.play_sound("point")
            self._nach_punkt(toward_player=False)

    def _bounce(self, paddle, nach_rechts):
        """Ball am Schläger reflektieren; Winkel hängt vom Treffpunkt ab."""
        treff = (self.ball_y + BALL_SIZE / 2) - (paddle.y + PADDLE_H / 2)
        treff /= (PADDLE_H / 2)

        speed = min(BALL_MAX_SPEED, abs(self.ball_vx) * 1.06 + 20)
        self.ball_vx = speed * (1 if nach_rechts else -1)
        self.ball_vy = speed * treff
        if nach_rechts:
            self.ball_x = paddle.right
        else:
            self.ball_x = paddle.left - BALL_SIZE
        self.play_sound("bounce")
        self.rumble(60)

    def _nach_punkt(self, toward_player):
        if self.player_score >= WIN_SCORE or self.ai_score >= WIN_SCORE:
            self.game_over = True
            self.play_sound("win" if self.player_score > self.ai_score else "gameover")
            self.rumble(200)
        else:
            self._serve(toward_player=toward_player)

    def draw(self):
        s = self.surface
        # Themen-Hintergrund (intern gecacht - Sterne/Aurora bleiben lebendig).
        ui.draw_background(s, self.width, self.height)

        # Gestrichelte Mittellinie im dezenten Rahmenton des Themes.
        for y in range(0, self.height, 28):
            pygame.draw.rect(s, ui.BORDER, (self.width // 2 - 2, y, 4, 16))

        pygame.draw.rect(s, COL_P1, (20, self.player_y, PADDLE_W, PADDLE_H),
                         border_radius=4)
        rechts_farbe = COL_P2 if self.multiplayer else ui.TEXT
        pygame.draw.rect(s, rechts_farbe,
                         (self.width - 20 - PADDLE_W, self.ai_y, PADDLE_W, PADDLE_H),
                         border_radius=4)

        pygame.draw.rect(s, COL_BALL, (self.ball_x, self.ball_y, BALL_SIZE, BALL_SIZE),
                         border_radius=3)

        # Spielstand mittig oben (rechtsbündig/linksbündig um die Mittellinie).
        ps = self.big_font.render(str(self.player_score), True, ui.TEXT)
        ais = self.big_font.render(str(self.ai_score), True, ui.TEXT)
        s.blit(ps, ps.get_rect(topright=(self.width // 2 - 44, 16)))
        s.blit(ais, (self.width // 2 + 44, 16))

        links = "P1" if self.multiplayer else t("pong.you")
        rechts = "P2" if self.multiplayer else t("pong.ai")
        s.blit(self.font.render(links, True, COL_P1), (34, 12))
        r_img = self.font.render(rechts, True, rechts_farbe)
        s.blit(r_img, (self.width - r_img.get_width() - 34, 12))

        if not self.game_over:
            self._draw_mode_hud()

        if self.game_over:
            if self.multiplayer:
                sieger = 1 if self.player_score > self.ai_score else 2
                text = t("common.player_wins", n=sieger)
                farbe = COL_P1 if sieger == 1 else COL_P2
            else:
                gewonnen = self.player_score > self.ai_score
                text = t("pong.won") if gewonnen else t("pong.lost")
                farbe = ui.GREEN if gewonnen else ui.RED
            self._draw_overlay(text, farbe, t("common.enter_restart"))

    def _draw_overlay(self, titel, farbe, hinweis):
        """Endstand-Box: transluzentes Themen-Panel mit farbigem Rahmen."""
        th = self.big_font.get_height()
        hh = self.font.get_height()
        bw = max(self.big_font.size(titel)[0], self.font.size(hinweis)[0]) + 80
        bh = th + hh + 58
        rect = pygame.Rect(0, 0, bw, bh)
        rect.center = (self.width // 2, self.height // 2)

        panel = pygame.Surface(rect.size, pygame.SRCALPHA)
        pygame.draw.rect(panel, (*ui.PANEL[:3], 235), panel.get_rect(), border_radius=16)
        pygame.draw.rect(panel, farbe, panel.get_rect(), 2, border_radius=16)
        self.surface.blit(panel, rect)

        img = self.big_font.render(titel, True, farbe)
        self.surface.blit(img, img.get_rect(midtop=(rect.centerx, rect.y + 20)))
        # Neustart-Hinweis sanft pulsieren lassen.
        hint_col = ui.mix(ui.TEXT_DIM, ui.TEXT, ui.pulse(2.0, 0.2, 1.0))
        img = self.font.render(hinweis, True, hint_col)
        self.surface.blit(img, img.get_rect(midtop=(rect.centerx, rect.y + 20 + th + 14)))

    def _draw_mode_hud(self):
        """Zeigt unten den Bewegungsmodus je Steuerung + die Umschalttasten."""
        s = self.surface
        y = self.height - self._small.get_height() - 8

        def zeile(scheme):
            modus = t("pong.hold") if self.hold[scheme] else t("pong.continuous")
            col = ui.GREEN if self.hold[scheme] else ui.TEXT_DIM
            return modus, col

        # Einzelspieler: beide Steuerungen gehören dir. Mehrspieler: 1=links, 2=rechts.
        m1, c1 = zeile("p1")
        t1 = self._small.render(t("pong.scheme1", mode=m1), True, c1)
        s.blit(t1, (10, y))

        m2, c2 = zeile("p2")
        t2 = self._small.render(t("pong.scheme2", mode=m2), True, c2)
        s.blit(t2, (self.width - t2.get_width() - 10, y))
