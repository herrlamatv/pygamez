# -*- coding: utf-8 -*-
"""
invaders.py
===========
Space Invaders (vereinfacht).

- Steuerung: Pfeil links/rechts (oder A/D) = bewegen, Leertaste = schiessen.
- Ein Block aus Aliens wandert seitlich, rueckt bei Randberuehrung nach unten
  und wird schneller, je weniger uebrig sind. Aliens schiessen gelegentlich
  zurueck.
- Treffer geben Punkte (obere Reihen mehr). Ist eine Welle geleert, kommt die
  naechste (schneller, tiefer). Erreichen Aliens den Spieler oder trifft ein
  Alien-Schuss -> ein Leben weniger; bei 0 Leben Game Over.
- Highscore wird gespeichert.
"""

import random
import pygame

from game_base import Game, InputEvent

COL_BG = (8, 10, 18)
COL_PLAYER = (110, 220, 140)
COL_BULLET = (240, 240, 120)
COL_EBULLET = (240, 120, 120)
COL_TEXT = (230, 230, 235)
COL_SHIELD = (90, 150, 200)

ALIEN_COLORS = [(235, 110, 110), (235, 170, 90), (110, 180, 235)]

PLAYER_W = 44
PLAYER_H = 16
PLAYER_SPEED = 300
PLAYER_Y_OFF = 40          # Abstand des Spielers vom unteren Rand

BULLET_SPEED = 480
EBULLET_SPEED = 220
SHOOT_COOLDOWN = 0.35      # Sekunden zwischen eigenen Schuessen

ALIEN_COLS = 8
ALIEN_ROWS = 4
ALIEN_W = 30
ALIEN_H = 20
ALIEN_GAP_X = 16
ALIEN_GAP_Y = 14


class InvadersGame(Game):
    name = "Invaders"
    highscore_key = "invaders"

    def reset(self):
        self.score = 0
        self.game_over = False

        self.lives = 3
        self.wave = 1
        self.player_x = self.width / 2 - PLAYER_W / 2
        self.move_dir = 0
        self._cooldown = 0.0
        self.bullets = []          # eigene Schuesse: [x, y]
        self.ebullets = []         # Alien-Schuesse: [x, y]

        self._spawn_wave()

    def _spawn_wave(self):
        """Baut den Alien-Block und die Schutzschilde neu auf."""
        self.aliens = []           # jeweils dict: x, y, row
        block_w = ALIEN_COLS * (ALIEN_W + ALIEN_GAP_X) - ALIEN_GAP_X
        start_x = (self.width - block_w) / 2
        start_y = 50
        for r in range(ALIEN_ROWS):
            for c in range(ALIEN_COLS):
                self.aliens.append({
                    "x": start_x + c * (ALIEN_W + ALIEN_GAP_X),
                    "y": start_y + r * (ALIEN_H + ALIEN_GAP_Y),
                    "row": r,
                })
        # Bewegung wird pro Welle etwas schneller.
        self.alien_dir = 1
        self.alien_speed = 26 + (self.wave - 1) * 10
        self.alien_drop = 16
        self._alien_shoot_timer = 0.0

        # Schutzschilde: einfache Rechtecke mit "Gesundheit".
        self.shields = []
        for i in range(4):
            sx = (i + 1) * self.width / 5 - 24
            self.shields.append({"x": sx, "y": self.height - 130,
                                 "w": 48, "h": 20, "hp": 6})

    # ----- Eingabe ------------------------------------------------------

    def handle_event(self, event):
        if event.kind != InputEvent.KEYDOWN:
            return

        if self.game_over:
            if event.key in ("Return", "space"):
                self.reset()
            return

        if self.is_action(event.key, "left"):
            self.move_dir = -1
        elif self.is_action(event.key, "right"):
            self.move_dir = 1
        elif self.is_action(event.key, "up") or self.is_action(event.key, "down"):
            self.move_dir = 0       # anhalten
        elif self.is_action(event.key, "action"):
            self._shoot()

    def _shoot(self):
        if self._cooldown <= 0:
            self.bullets.append([self.player_x + PLAYER_W / 2,
                                 self.height - PLAYER_Y_OFF - PLAYER_H])
            self._cooldown = SHOOT_COOLDOWN
            self.play_sound("shoot")

    # ----- Spiellogik ---------------------------------------------------

    def update(self, dt):
        if self.game_over:
            return

        self._cooldown = max(0.0, self._cooldown - dt)

        # Spieler bewegen
        self.player_x += self.move_dir * PLAYER_SPEED * dt
        self.player_x = max(0, min(self.width - PLAYER_W, self.player_x))

        self._update_aliens(dt)
        self._update_bullets(dt)
        self._update_ebullets(dt)

        # Welle geleert -> naechste Welle.
        if not self.aliens:
            self.wave += 1
            self.bullets.clear()
            self.ebullets.clear()
            self._spawn_wave()

    def _update_aliens(self, dt):
        # Geschwindigkeit steigt, je weniger Aliens uebrig sind.
        faktor = 1.0 + (ALIEN_COLS * ALIEN_ROWS - len(self.aliens)) * 0.03
        dx = self.alien_dir * self.alien_speed * faktor * dt

        links = min(a["x"] for a in self.aliens)
        rechts = max(a["x"] + ALIEN_W for a in self.aliens)

        # Rand erreicht -> Richtung wechseln und eine Reihe nach unten.
        if (self.alien_dir > 0 and rechts + dx >= self.width - 8) or \
           (self.alien_dir < 0 and links + dx <= 8):
            self.alien_dir *= -1
            for a in self.aliens:
                a["y"] += self.alien_drop
        else:
            for a in self.aliens:
                a["x"] += dx

        # Aliens schiessen zufaellig aus der jeweils untersten Reihe.
        self._alien_shoot_timer -= dt
        if self._alien_shoot_timer <= 0:
            self._alien_shoot_timer = random.uniform(0.4, 1.1)
            unterste = {}
            for a in self.aliens:
                col = round(a["x"])
                if col not in unterste or a["y"] > unterste[col]["y"]:
                    unterste[col] = a
            if unterste:
                schuetze = random.choice(list(unterste.values()))
                self.ebullets.append([schuetze["x"] + ALIEN_W / 2,
                                      schuetze["y"] + ALIEN_H])

        # Aliens erreichen die Spielerhoehe -> Leben verlieren / Game Over.
        grenze = self.height - PLAYER_Y_OFF - PLAYER_H
        if any(a["y"] + ALIEN_H >= grenze for a in self.aliens):
            self._hit_player()
            # Block wieder etwas nach oben, damit das Spiel weitergehen kann.
            for a in self.aliens:
                a["y"] -= self.alien_drop * 3

    def _update_bullets(self, dt):
        player_rect = self._player_rect()
        neu = []
        for b in self.bullets:
            b[1] -= BULLET_SPEED * dt
            if b[1] < 0:
                continue
            if self._bullet_hits_shield(b):
                continue
            getroffen = self._bullet_hits_alien(b)
            if not getroffen:
                neu.append(b)
        self.bullets = neu

    def _bullet_hits_alien(self, b):
        for a in self.aliens:
            if a["x"] <= b[0] <= a["x"] + ALIEN_W and \
               a["y"] <= b[1] <= a["y"] + ALIEN_H:
                self.aliens.remove(a)
                # Obere Reihen geben mehr Punkte.
                self.score += (ALIEN_ROWS - a["row"]) * 10
                self.play_sound("explode")
                return True
        return False

    def _update_ebullets(self, dt):
        player_rect = self._player_rect()
        neu = []
        for b in self.ebullets:
            b[1] += EBULLET_SPEED * dt
            if b[1] > self.height:
                continue
            if self._bullet_hits_shield(b):
                continue
            if player_rect.collidepoint(b[0], b[1]):
                self._hit_player()
                continue
            neu.append(b)
        self.ebullets = neu

    def _bullet_hits_shield(self, b):
        """Prueft Schild-Treffer; verringert dessen HP. True bei Treffer."""
        for sh in self.shields:
            if sh["hp"] > 0 and sh["x"] <= b[0] <= sh["x"] + sh["w"] and \
               sh["y"] <= b[1] <= sh["y"] + sh["h"]:
                sh["hp"] -= 1
                return True
        return False

    def _player_rect(self):
        return pygame.Rect(self.player_x, self.height - PLAYER_Y_OFF - PLAYER_H,
                           PLAYER_W, PLAYER_H)

    def _hit_player(self):
        self.lives -= 1
        self.ebullets.clear()
        self.play_sound("hit")
        self.rumble(180)
        if self.lives <= 0:
            self.lives = 0
            self.game_over = True
            self.play_sound("gameover")

    # ----- Zeichnen -----------------------------------------------------

    def draw(self):
        s = self.surface
        s.fill(COL_BG)

        # Schutzschilde (verblassen mit sinkender HP)
        for sh in self.shields:
            if sh["hp"] <= 0:
                continue
            alpha = int(80 + (sh["hp"] / 6) * 175)
            farbe = (COL_SHIELD[0], COL_SHIELD[1], min(255, alpha))
            pygame.draw.rect(s, farbe, (sh["x"], sh["y"], sh["w"], sh["h"]),
                             border_radius=4)

        # Aliens
        for a in self.aliens:
            farbe = ALIEN_COLORS[a["row"] % len(ALIEN_COLORS)]
            pygame.draw.rect(s, farbe, (a["x"], a["y"], ALIEN_W, ALIEN_H),
                             border_radius=5)
            # kleine "Augen"
            pygame.draw.rect(s, COL_BG, (a["x"] + 7, a["y"] + 7, 4, 4))
            pygame.draw.rect(s, COL_BG, (a["x"] + ALIEN_W - 11, a["y"] + 7, 4, 4))

        # Spieler
        pr = self._player_rect()
        pygame.draw.rect(s, COL_PLAYER, pr, border_radius=4)
        pygame.draw.rect(s, COL_PLAYER,
                         (pr.centerx - 3, pr.y - 8, 6, 8))   # Kanone

        # Schuesse
        for b in self.bullets:
            pygame.draw.rect(s, COL_BULLET, (b[0] - 2, b[1] - 8, 4, 10))
        for b in self.ebullets:
            pygame.draw.rect(s, COL_EBULLET, (b[0] - 2, b[1], 4, 10))

        # HUD
        s.blit(self.font.render(f"Punkte: {self.score}", True, COL_TEXT), (10, 8))
        welle = self.font.render(f"Welle: {self.wave}", True, COL_TEXT)
        s.blit(welle, (self.width // 2 - welle.get_width() // 2, 8))
        leben = self.font.render(f"Leben: {self.lives}", True, COL_TEXT)
        s.blit(leben, (self.width - leben.get_width() - 10, 8))

        if self.game_over:
            self.draw_center_text("GAME OVER", self.big_font, (235, 110, 110), -20)
            self.draw_center_text("Enter = Neustart", self.font, COL_TEXT, 30)
