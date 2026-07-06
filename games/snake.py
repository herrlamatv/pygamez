# -*- coding: utf-8 -*-
"""
snake.py
========
Snake - Einzelspieler oder Mehrspieler (2 Schlangen an einer Tastatur).

- Steuerung ueber die in den Optionen belegten Tasten (Standard: Spieler 1 = WASD,
  Spieler 2 = Pfeiltasten). Im Einzelspieler steuern beide Belegungen dieselbe
  Schlange, im Mehrspieler je eine.
- Vor dem Spiel gibt es einen kleinen Einstell-Screen (SETUP):
    * Waende (durchgehen): AN  -> man laeuft durch die Wand und kommt auf der
      Gegenseite wieder heraus. Im Spiel zeigt "WLS" in GRUEN an, dass das an ist.
    * Bonus-Aepfel (1-2): AN  -> jeder Apfel zaehlt zufaellig 1 oder 2 (Schnitt ~1,5).
  Die Einstellungen werden dauerhaft in settings.json ("snake") gespeichert.
- Einzelspieler-Extras:
    * Apfel-Zaehler (wird auch beim Game Over angezeigt).
    * Prestige (siehe prestige.py): mit P steigt man eine Stufe auf, wenn man genug
      Aepfel gesammelt hat. Jede Stufe KOSTET Aepfel UND Koerperlaenge (beides wird
      verloren) und wird mit steigender Stufe teurer. Dafuer waechst die Schlange
      pro Apfel um (1 + Level) Bloecke und die Punkte verdoppeln sich je Stufe.
      Das erreichte Level wird im Spiel als roemische Zahl angezeigt (Prestige I..X).
- Highscore wird gespeichert und beim Game Over eingeblendet (siehe main.py).
"""

import random
import pygame

import highscore
import prestige
import settings as settings_mod
from game_base import Game, InputEvent

CELL = 20                       # Kantenlaenge einer Rasterzelle in Pixeln
MOVE_INTERVAL = 0.11            # Sekunden pro Schritt (Spielgeschwindigkeit)
MIN_LENGTH = 3                  # so kurz darf eine Schlange durch Prestige max. werden

# Spielzustaende
SETUP, PLAY = "setup", "play"

COL_BG = (15, 15, 25)
COL_GRID = (25, 25, 40)
COL_FOOD = (240, 90, 90)
COL_TEXT = (230, 230, 230)
COL_DIM = (150, 158, 176)
COL_BTN = (44, 50, 66)
COL_BTN_ON = (60, 120, 80)
COL_WLS_ON = (90, 230, 130)     # WLS gruen = durch die Waende gehen ist AN
COL_WLS_OFF = (105, 105, 120)   # WLS grau  = feste Waende
COL_MULT = (255, 210, 90)       # Multiplikator / Prestige (gold)

# Farben je Schlange: (Koerper, Kopf)
SNAKE_COLORS = [
    ((80, 220, 120), (140, 255, 170)),   # Spieler 1 (gruen)
    ((90, 160, 240), (150, 200, 255)),   # Spieler 2 (blau)
]


class _Snake:
    """Zustand einer einzelnen Schlange (Koerper: Kopf am Listenende)."""

    def __init__(self, body, direction, player):
        self.body = list(body)
        self.direction = direction
        self.next_direction = direction
        self.player = player            # "p1" / "p2" (nur fuer die Anzeige/Steuerung)
        self.alive = True
        self.score = 0
        self.grow = 0                   # ausstehende Wachstums-Bloecke (Prestige)


class SnakeGame(Game):
    name = "Snake"
    highscore_key = "snake"
    supports_multiplayer = True

    def reset(self):
        self.score = 0
        self.game_over = False
        self.winner = None

        self.cols = self.width // CELL
        self.rows = self.height // CELL

        # Snake-spezifische Einstellungen aus den globalen Settings laden.
        snk = self.settings.get("snake", {}) if isinstance(self.settings, dict) else {}
        self.wrap = bool(snk.get("wrap", False))
        self.bonus = bool(snk.get("bonus_apple", False))

        self._small = pygame.font.SysFont("consolas", 16)
        self.highscore = highscore.load_highscores().get(self.highscore_key, 0)

        self._build_setup_layout()
        self._reset_run_stats()
        self._new_board()
        # Vor dem Spiel zuerst der Einstell-Screen.
        self.state = SETUP

    def _reset_run_stats(self):
        """Setzt die Lauf-Statistik (Aepfel/Prestige) zurueck."""
        self.apples_total = 0     # insgesamt eingesammelte Aepfel (Anzeige)
        self.apples_bank = 0      # verfuegbare Aepfel fuer das naechste Prestige
        self.prestige = 0         # erreichtes Prestige-Level (0 = keins)

    def _new_board(self):
        """Baut die Schlange(n) und setzt das erste Futter."""
        cy = self.rows // 2
        self.snakes = []
        if self.multiplayer:
            # Zwei Schlangen, versetzt gestartet, beide nach rechts.
            self.snakes.append(_Snake(
                [(2, cy - 3), (3, cy - 3), (4, cy - 3)], (1, 0), "p1"))
            self.snakes.append(_Snake(
                [(2, cy + 3), (3, cy + 3), (4, cy + 3)], (1, 0), "p2"))
        else:
            cx = self.cols // 2
            self.snakes.append(_Snake(
                [(cx - 2, cy), (cx - 1, cy), (cx, cy)], (1, 0), "p1"))
        self._timer = 0.0
        self._place_food()

    def _start_play(self):
        """Startet eine neue Runde mit den aktuellen Einstellungen."""
        self.score = 0
        self.game_over = False
        self.winner = None
        self._reset_run_stats()
        self._new_board()
        self.state = PLAY

    def _place_food(self):
        """Setzt Futter auf eine freie Zelle (keine Schlange)."""
        belegt = set()
        for sn in self.snakes:
            belegt |= set(sn.body)
        frei = [(x, y) for x in range(self.cols) for y in range(self.rows)
                if (x, y) not in belegt]
        self.food = random.choice(frei) if frei else None

    # ----- Einstell-Screen ----------------------------------------------

    def _build_setup_layout(self):
        """Klickbare Bereiche fuer den Einstell-Screen berechnen."""
        cx = self.width // 2
        bw, bh, gap = min(400, self.width - 60), 46, 16
        y0 = max(120, self.height // 2 - 90)
        self.wrap_rect = pygame.Rect(cx - bw // 2, y0, bw, bh)
        self.bonus_rect = pygame.Rect(cx - bw // 2, y0 + (bh + gap), bw, bh)
        self.start_rect = pygame.Rect(cx - 90, y0 + 2 * (bh + gap) + 16, 180, 50)

    def _toggle_setting(self, key):
        """Schaltet eine Snake-Option um und speichert sie dauerhaft."""
        snk = self.settings.setdefault("snake", {}) if isinstance(self.settings, dict) else {}
        neu = not snk.get(key, False)
        snk[key] = neu
        if key == "wrap":
            self.wrap = neu
        elif key == "bonus_apple":
            self.bonus = neu
        if isinstance(self.settings, dict):
            settings_mod.save_settings(self.settings)
        self.play_sound("select")

    def _handle_setup_event(self, event):
        if event.kind == InputEvent.KEYDOWN:
            if event.key in ("w", "W"):
                self._toggle_setting("wrap")
            elif event.key in ("b", "B"):
                self._toggle_setting("bonus_apple")
            elif event.key in ("Return", "space"):
                self.play_sound("click")
                self._start_play()
        elif event.kind == InputEvent.MOUSEDOWN:
            p = event.pos
            if self.wrap_rect.collidepoint(p):
                self._toggle_setting("wrap")
            elif self.bonus_rect.collidepoint(p):
                self._toggle_setting("bonus_apple")
            elif self.start_rect.collidepoint(p):
                self.play_sound("click")
                self._start_play()

    def _draw_setup(self):
        s = self.surface
        s.fill(COL_BG)

        title = self.big_font.render("SNAKE", True, COL_TEXT)
        s.blit(title, title.get_rect(center=(self.width // 2, 56)))
        modus = "Mehrspieler" if self.multiplayer else "Einzelspieler"
        sub = self._small.render(f"{modus}   -   Highscore: {self.highscore}",
                                 True, COL_DIM)
        s.blit(sub, sub.get_rect(center=(self.width // 2, 92)))

        self._draw_setup_toggle(self.wrap_rect, "Waende: durchgehen", self.wrap)
        self._draw_setup_toggle(self.bonus_rect, "Bonus-Aepfel (1-2)", self.bonus)

        pygame.draw.rect(s, COL_BTN_ON, self.start_rect, border_radius=10)
        st = self.font.render("START", True, COL_TEXT)
        s.blit(st, st.get_rect(center=self.start_rect.center))

        hint = self._small.render(
            "W = Waende   B = Bonus   Klick oder Enter = Start",
            True, COL_DIM)
        s.blit(hint, hint.get_rect(center=(self.width // 2, self.height - 40)))
        info = self._small.render(
            "Waende AN -> 'WLS' wird im Spiel gruen angezeigt (man laeuft durch).",
            True, COL_DIM)
        s.blit(info, info.get_rect(center=(self.width // 2, self.height - 18)))

    def _draw_setup_toggle(self, rect, label, an):
        s = self.surface
        pygame.draw.rect(s, COL_BTN_ON if an else COL_BTN, rect, border_radius=8)
        pygame.draw.rect(s, COL_DIM, rect, 1, border_radius=8)
        lab = self.font.render(label, True, COL_TEXT)
        s.blit(lab, (rect.x + 16, rect.centery - lab.get_height() // 2))
        wert = "AN" if an else "AUS"
        col = COL_WLS_ON if an else COL_DIM
        img = self.font.render(f"< {wert} >", True, col)
        s.blit(img, (rect.right - img.get_width() - 16,
                     rect.centery - img.get_height() // 2))

    # ----- Eingabe ------------------------------------------------------

    def handle_event(self, event):
        if self.state == SETUP:
            self._handle_setup_event(event)
            return

        if event.kind != InputEvent.KEYDOWN:
            return

        if self.game_over:
            if event.key in ("Return", "space"):
                self._start_play()
            return

        # Prestige (nur Einzelspieler): 10 Aepfel -> x2 Multiplikator.
        if event.key in ("p", "P"):
            self._try_prestige()
            return

        if self.multiplayer:
            self._turn(self.snakes[0], event.key, "p1")
            self._turn(self.snakes[1], event.key, "p2")
        else:
            # Einzelspieler: beide Belegungen steuern die eine Schlange.
            self._turn(self.snakes[0], event.key, None)

    def _turn(self, sn, key, player):
        """Setzt die gepufferte Richtung, verbietet aber 180-Grad-Wenden."""
        dx, dy = sn.direction
        if self.is_action(key, "up", player) and dy == 0:
            sn.next_direction = (0, -1)
        elif self.is_action(key, "down", player) and dy == 0:
            sn.next_direction = (0, 1)
        elif self.is_action(key, "left", player) and dx == 0:
            sn.next_direction = (-1, 0)
        elif self.is_action(key, "right", player) and dx == 0:
            sn.next_direction = (1, 0)

    def _can_prestige(self):
        """(req, ok) fuer die naechste Stufe: Anforderung und ob sie erfuellt ist."""
        if self.multiplayer:
            return None, False
        req = prestige.next_requirement(self.prestige)
        if req is None:
            return None, False
        sn = self.snakes[0]
        genug_aepfel = self.apples_bank >= req["apples"]
        genug_laenge = len(sn.body) - req["length"] >= MIN_LENGTH
        return req, (genug_aepfel and genug_laenge)

    def _try_prestige(self):
        """Steigt eine Prestige-Stufe auf - kostet Aepfel UND Koerperlaenge."""
        if self.game_over:
            return
        req, ok = self._can_prestige()
        if not ok:
            return
        # Aepfel und Laenge verlieren.
        self.apples_bank -= req["apples"]
        sn = self.snakes[0]
        del sn.body[:req["length"]]        # Segmente vom Schwanz entfernen
        sn.grow = 0                         # ausstehendes Wachstum verfaellt
        self.prestige += 1
        self.play_sound("select")
        self.rumble(120)

    # ----- Spiellogik ---------------------------------------------------

    def update(self, dt):
        if self.state != PLAY or self.game_over:
            return

        self._timer += dt
        if self._timer < MOVE_INTERVAL:
            return
        self._timer -= MOVE_INTERVAL

        # 1) Neue Kopfpositionen aller lebenden Schlangen bestimmen (mit Wrap).
        neue_koepfe = {}
        for i, sn in enumerate(self.snakes):
            if not sn.alive:
                continue
            sn.direction = sn.next_direction
            hx, hy = sn.body[-1]
            nx, ny = hx + sn.direction[0], hy + sn.direction[1]
            if self.wrap:
                # Durch die Wand: auf der Gegenseite wieder heraus.
                nx %= self.cols
                ny %= self.rows
            neue_koepfe[i] = (nx, ny)

        tot = set()

        # 2) Wandkollision - nur bei festen Waenden (wrap aus).
        if not self.wrap:
            for i, (nx, ny) in neue_koepfe.items():
                if nx < 0 or nx >= self.cols or ny < 0 or ny >= self.rows:
                    tot.add(i)

        # 3) Kopf-an-Kopf (beide Schlangen sterben)
        for i in neue_koepfe:
            for j in neue_koepfe:
                if i < j and neue_koepfe[i] == neue_koepfe[j]:
                    tot.add(i)
                    tot.add(j)

        # 4) Koerperkollision: Zellen, die nach dem Zug belegt bleiben.
        #    Der Schwanz rueckt nach - AUSSER die Schlange waechst diesen Zug
        #    (weil sie frisst oder noch Wachstum aus einem Apfel aussteht).
        belegt = set()
        for i, sn in enumerate(self.snakes):
            if not sn.alive:
                continue
            waechst = (neue_koepfe.get(i) == self.food) or (sn.grow > 0)
            koerper = sn.body if waechst else sn.body[1:]
            belegt |= set(koerper)
        for i, kopf in neue_koepfe.items():
            if kopf in belegt:
                tot.add(i)

        # 5) Bewegung anwenden bzw. Tod vermerken.
        gefressen = False
        for i, sn in enumerate(self.snakes):
            if not sn.alive:
                continue
            if i in tot:
                sn.alive = False
                continue
            kopf = neue_koepfe[i]
            sn.body.append(kopf)
            if kopf == self.food:
                # Bonus: ein Apfel zaehlt zufaellig 1 oder 2 (im Schnitt ~1,5).
                gain = random.randint(1, 2) if self.bonus else 1
                # Wachstum pro Apfel steigt mit dem Prestige-Level.
                sn.grow += gain * prestige.blocks_per_apple(self.prestige)
                if self.multiplayer:
                    sn.score += gain * 10
                else:
                    self.apples_total += gain
                    self.apples_bank += gain
                    sn.score += gain * 10 * prestige.score_multiplier(self.prestige)
                gefressen = True
            # Schwanz behalten, solange Wachstum aussteht - sonst nachruecken.
            if sn.grow > 0:
                sn.grow -= 1
            else:
                sn.body.pop(0)

        if gefressen:
            self.play_sound("eat")
            self._place_food()

        self._check_end(tot)

    def _check_end(self, tot):
        """Prueft Spielende und ermittelt ggf. den Gewinner."""
        if self.multiplayer:
            self.score = max(sn.score for sn in self.snakes)
            lebende = [i for i, sn in enumerate(self.snakes) if sn.alive]
            if len(lebende) <= 1 and (tot or len(lebende) < len(self.snakes)):
                self.game_over = True
                if len(lebende) == 1:
                    self.winner = lebende[0]
                else:
                    # beide tot: hoehere Punktzahl gewinnt (sonst Unentschieden)
                    s0, s1 = self.snakes[0].score, self.snakes[1].score
                    self.winner = 0 if s0 > s1 else (1 if s1 > s0 else None)
                self._end_effects()
        else:
            self.score = self.snakes[0].score
            if not self.snakes[0].alive:
                self.game_over = True
                self._end_effects()

    def _end_effects(self):
        self.highscore = max(self.highscore, self.score)
        self.play_sound("gameover")
        self.rumble(200)

    # ----- Zeichnen -----------------------------------------------------

    def draw(self):
        if self.state == SETUP:
            self._draw_setup()
            return

        s = self.surface
        s.fill(COL_BG)

        for x in range(0, self.cols * CELL, CELL):
            pygame.draw.line(s, COL_GRID, (x, 0), (x, self.rows * CELL))
        for y in range(0, self.rows * CELL, CELL):
            pygame.draw.line(s, COL_GRID, (0, y), (self.cols * CELL, y))

        if self.food:
            fx, fy = self.food
            pygame.draw.rect(s, COL_FOOD,
                             (fx * CELL + 2, fy * CELL + 2, CELL - 4, CELL - 4),
                             border_radius=6)

        for idx, sn in enumerate(self.snakes):
            koerper, kopf = SNAKE_COLORS[idx % len(SNAKE_COLORS)]
            if not sn.alive:
                koerper = tuple(c // 2 for c in koerper)   # abgedunkelt, wenn tot
                kopf = koerper
            for i, (x, y) in enumerate(sn.body):
                farbe = kopf if i == len(sn.body) - 1 else koerper
                pygame.draw.rect(s, farbe,
                                 (x * CELL + 1, y * CELL + 1, CELL - 2, CELL - 2),
                                 border_radius=4)

        self._draw_hud()

        if self.game_over:
            self._draw_game_over()

    def _draw_hud(self):
        s = self.surface

        # WLS-Anzeige oben mittig: gruen = durch die Waende gehen ist AN.
        wls_col = COL_WLS_ON if self.wrap else COL_WLS_OFF
        wls = self.font.render("WLS", True, wls_col)
        s.blit(wls, wls.get_rect(midtop=(self.width // 2, 8)))

        if self.multiplayer:
            p1 = self.font.render(f"P1: {self.snakes[0].score}", True,
                                  SNAKE_COLORS[0][1])
            p2 = self.font.render(f"P2: {self.snakes[1].score}", True,
                                  SNAKE_COLORS[1][1])
            s.blit(p1, (10, 8))
            s.blit(p2, (self.width - p2.get_width() - 10, 8))
            return

        # Einzelspieler: Punkte, Aepfel, Prestige + Highscore.
        s.blit(self.font.render(f"Punkte: {self.score}", True, COL_TEXT), (10, 8))
        s.blit(self._small.render(f"Aepfel: {self.apples_total}   Bank: {self.apples_bank}",
                                  True, COL_FOOD), (10, 34))
        if self.prestige > 0:
            blocks = prestige.blocks_per_apple(self.prestige)
            info = self._small.render(
                f"Prestige {prestige.roman(self.prestige)}   {blocks} Bloecke/Apfel",
                True, COL_MULT)
            s.blit(info, (10, 54))

        best = self._small.render(f"Best: {self.highscore}", True, COL_DIM)
        s.blit(best, (self.width - best.get_width() - 10, 10))

        # Naechste Prestige-Stufe samt Anforderung unten anzeigen.
        if not self.game_over:
            self._draw_next_prestige()

    def _draw_next_prestige(self):
        s = self.surface
        req, ok = self._can_prestige()
        if req is None:
            txt = f"MAX PRESTIGE {prestige.roman(self.prestige)} erreicht"
            col = COL_MULT
        else:
            txt = (f"Prestige {req['roman']}:  {req['apples']} Aepfel   "
                   f"-{req['length']} Laenge")
            if ok:
                txt += "   ->  P druecken!"
                col = COL_MULT
            else:
                col = COL_DIM
        img = self._small.render(txt, True, col)
        s.blit(img, img.get_rect(midbottom=(self.width // 2, self.height - 8)))

    def _draw_game_over(self):
        if self.multiplayer:
            if self.winner is None:
                text, farbe = "UNENTSCHIEDEN", COL_TEXT
            else:
                text = f"SPIELER {self.winner + 1} GEWINNT"
                farbe = SNAKE_COLORS[self.winner][1]
            self.draw_center_text(text, self.big_font, farbe, -30)
            self.draw_center_text("Enter = Neustart", self.font, COL_TEXT, 20)
            return

        # Einzelspieler: Game Over mit Apfel-Anzahl (und ggf. Prestige-Stufe).
        self.draw_center_text("GAME OVER", self.big_font, COL_FOOD, -60)
        self.draw_center_text(f"Aepfel eingesammelt: {self.apples_total}",
                              self.font, COL_FOOD, -14)
        if self.prestige > 0:
            self.draw_center_text(f"Prestige {prestige.roman(self.prestige)}",
                                  self.font, COL_MULT, 16)
        self.draw_center_text("Enter = Neustart", self.font, COL_TEXT, 46)
