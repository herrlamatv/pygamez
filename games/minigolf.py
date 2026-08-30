# -*- coding: utf-8 -*-
"""
minigolf.py
===========
Minigolf - 360 Bahnen in 40 Kursen, allein oder zu zweit lokal.

Kurse (im Setup wählbar, wird gespeichert):
  - Classic : 9 handgebaute, freundliche Bahnen (Par 2-4), sanfter Einstieg.
  - Pro     : 9 handgebaute Bahnen mit Inselgrün, Doppelmühle und Wanderblöcken.
  - Tour    : 38 seed-erzeugte Kurse zu je 9 Bahnen (342 Bahnen) mit steigender
              Schwierigkeit - Kurs 7, Bahn 3 sieht überall gleich aus, gespeichert
              werden muss dafür nichts (siehe minigolf_gen.py).
  - Random  : 9 zufällig gezogene Bahnen aus allen Kursen, zufällig gespiegelt.

Die Physik läuft - wie beim Billard - in Teilschritten mit Reibung, damit
nichts ruckt und schnelle Bälle nicht durch Banden tunneln. Untergründe
bremsen unterschiedlich (Grün/Sand), Rampen beschleunigen, Wasser kostet einen
Strafschlag, Gummipuffer geben Tempo zurück, Windmühlen und Wanderblöcke
verlangen Timing.

Steuerung: Maus bewegt die Ziellinie, linke Maustaste gedrückt halten lädt die
Schlagstärke, Loslassen schlägt. Die rechte Maustaste hält die Stärke fest,
solange sie gedrückt bleibt. R bricht einen geladenen Schlag ab, ohne zu
putten. Alternativ Pfeile links/rechts zielen, hoch/runter Stärke, Leertaste
schlägt. G blendet die Ziellinie um, Z das Autoziel, P das Aufnehmen, F setzt
die laufende Bahn zurück (Schläge auf 0, gleiche Bahn).

Autoziel (Setup, Standard AN): vor jedem Schlag dreht sich der Schläger von
selbst zum Loch. AUS heißt, dass die zuletzt gewählte Richtung stehen bleibt -
gezielt wird dann komplett selbst; nur ganz zu Beginn einer Bahn zeigt der
Schläger neutral bahnaufwärts.

Stärke-Sperre (rechte Maustaste halten): friert den Ladebalken genau da ein, wo
er gerade steht. Der Balken wird golden, zeigt die Prozentzahl und pulst - so
wartet man mit fertig geladenem Schlag auf die Lücke in der Windmühle oder auf
den Wanderblock und puttet im richtigen Moment. Loslassen lädt weiter. Gesperrt
bleibt die Stärke auch über den Schlag hinaus: der nächste Linksklick setzt sie
dann nicht auf 5% zurück, sondern schlägt exakt mit dem gehaltenen Wert - auch
eine mit Pfeil hoch/runter eingestellte Stärke lässt sich so festnageln.

Am Rundenende führt der Weiter-Knopf zum NÄCHSTEN Kurs (Classic -> Pro ->
Tour 1 -> Tour 2 -> ...), damit sich nicht immer derselbe Neuner-Satz
wiederholt; daneben stehen Nochmal (gleicher Kurs) und Setup.

Punkte (Highscore) = Summe der Bahnpunkte; je Bahn gibt es mehr Punkte, je
weiter unter Par gespielt wird (Hole-in-One extra).
"""

import math
import random

import pygame

import replay
import settings as settings_mod
import store
import ui
from game_base import Game, InputEvent
from i18n import t

from . import minigolf_draw as draw
from . import minigolf_gen as gen
from .minigolf_gen import CW, CH, BORDER, HOLES_PER_ROUND, make_hole as _hole
# Die Farben des Platzes stehen in minigolf_draw.py - dort wird gezeichnet,
# und zwar für Spiel UND Bahn-Editor. Hier stehen nur die Namen, die auch
# außerhalb des Platzes gebraucht werden (Ball, Ziellinie, Stärke-Sperre).
from .minigolf_draw import COL_AIM, COL_BALL, COL_GREEN, COL_LOCK

# ------------------------------------------------------------- Platz / Physik
# CW/CH/BORDER kommen aus minigolf_gen.py (dort steht die einzige Definition).
# CW/CH sind nur noch die STANDARD-Maße: eigene Bahnen aus dem Editor bringen
# ihre eigene Größe mit (hole["w"]/hole["h"], siehe self.cw/self.ch).
BR = gen.BALL_R              # Ballradius
CUP_R = 3.0                  # Lochradius
ARM_W = 1.5                  # halbe Breite eines Mühlenflügels

FRIC_GREEN = 1.15            # Rollreibung je Sekunde
FRIC_SAND = 4.60
FRIC_ICE = 0.25              # Eis: der Ball läuft fast ewig
FRIC_STICKY = 12.0           # Klebefeld: bleibt sofort liegen
WALL_E = 0.72                # Bandenrestitution
BUMP_E = 1.18                # Gummipuffer geben Tempo zurück
STOP_EPS = 2.2               # darunter gilt der Ball als still
MAX_SPEED = 205.0            # maximale Schlaggeschwindigkeit
CAPTURE_SPEED = 74.0         # darüber springt der Ball über das Loch
MAX_SHOT_TIME = 14.0
MAX_STROKES = 8              # danach wird die Bahn mit Höchstwert beendet

# --- Kennwerte der acht Editor-Hindernisse ---------------------------------
TUNNEL_KEEP = 0.95           # Tempo, das ein Rohr durchlässt
TUNNEL_COOLDOWN = 0.35       # Sekunden Sperre, damit es nicht zurückspringt
BOOST_COOLDOWN = 0.25        # ein Schubfeld feuert nicht in jedem Teilschritt
SPIN_PUSH = 0.55             # wie stark eine Drehscheibe mitnimmt
JUMP_MIN_SPEED = 30.0        # darunter ist der Ball zu langsam zum Abheben

SETUP, PLAY, HOLE_DONE, OVER, EDIT = ("setup", "play", "holedone", "over",
                                      "edit")
# Rand über der Überschrift und unter der Tastenzeile im Rundenende-Banner.
OVER_PAD = 14
# Die fünfte Wahl "ugc" spielt die selbst gebauten Bahnen (siehe MAPS-Reiter).
COURSES = ["classic", "pro", "tour", "random", "ugc"]
# Reiter des Vorbereitungs-Screens.
TABS = ("play", "maps")


# --------------------------------------------------------- Kurs 1: Classic
HOLES_CLASSIC = [
    # 1 - gerade Bahn mit Trichter
    _hole(2, (50, 140), (50, 26),
          walls=[(20, 60, 14, 8), (66, 60, 14, 8)]),
    # 2 - Dogleg nach rechts
    _hole(3, (24, 140), (76, 30),
          walls=[(38, 40, 10, 74), (60, 96, 26, 8)],
          sand=[(58, 118, 26, 14)]),
    # 3 - Mittelblock mit Sandgürtel
    _hole(3, (50, 142), (50, 22),
          walls=[(38, 66, 24, 20)],
          sand=[(14, 62, 20, 28), (66, 62, 20, 28)]),
    # 4 - Teich links, schmale Passage rechts
    _hole(3, (26, 142), (76, 26),
          water=[(12, 56, 44, 44)],
          walls=[(66, 74, 8, 50)]),
    # 5 - S-Kurve
    _hole(4, (20, 144), (80, 24),
          walls=[(30, 108, 62, 8), (8, 66, 62, 8), (30, 30, 46, 8)]),
    # 6 - Gummipuffer-Feld
    _hole(3, (50, 142), (50, 22),
          bumpers=[(30, 96, 6), (70, 96, 6), (50, 66, 7), (30, 44, 5),
                   (70, 44, 5)]),
    # 7 - Windmühle
    _hole(4, (50, 144), (50, 22),
          walls=[(6, 84, 30, 8), (64, 84, 30, 8)],
          mills=[(50, 88, 13, 2, 1.5)]),
    # 8 - Steigung mit Sandfang
    _hole(3, (50, 144), (50, 20),
          slopes=[(20, 50, 60, 56, 0.0, 34.0)],
          sand=[(20, 112, 24, 16), (56, 112, 24, 16)]),
    # 9 - Wanderblock vor dem Grün
    _hole(4, (26, 142), (74, 26),
          walls=[(44, 34, 8, 46), (44, 106, 8, 40)],
          movers=[(20, 80, 24, 8, 40, 0, 26)],
          bumpers=[(78, 96, 6)]),
]

# ------------------------------------------------------------ Kurs 2: Pro
HOLES_PRO = [
    # 10 - Inselgrün mit schmalem Hals
    _hole(3, (50, 144), (50, 36),
          water=[(10, 14, 80, 12), (10, 26, 26, 22), (64, 26, 26, 22),
                 (10, 48, 34, 12), (56, 48, 34, 12)],
          walls=[(26, 62, 14, 6), (60, 62, 14, 6)],
          slopes=[(20, 72, 30, 40, 11.0, -8.0),
                  (50, 72, 30, 40, -11.0, -8.0)]),
    # 11 - Doppelmühle im Korridor
    _hole(4, (50, 146), (50, 18),
          walls=[(28, 20, 6, 100), (66, 20, 6, 100)],
          mills=[(50, 106, 15, 2, -1.8), (50, 56, 15, 2, 2.2)]),
    # 12 - Puffer-Tunnel
    _hole(4, (18, 142), (82, 26),
          walls=[(34, 100, 8, 46), (58, 46, 8, 46)],
          bumpers=[(50, 122, 7), (24, 74, 6), (76, 74, 6), (50, 34, 6)],
          sand=[(60, 116, 26, 20)]),
    # 13 - Zickzack-Labyrinth
    _hole(5, (14, 146), (86, 20),
          walls=[(24, 118, 70, 7), (6, 92, 70, 7), (24, 66, 70, 7),
                 (6, 40, 70, 7)]),
    # 14 - Wanderschleusen
    _hole(4, (50, 146), (50, 18),
          walls=[(6, 108, 34, 7), (60, 108, 34, 7),
                 (6, 56, 34, 7), (60, 56, 34, 7)],
          movers=[(40, 108, 12, 7, 8, 0, 14), (48, 56, 12, 7, -8, 0, 12)]),
    # 15 - Sandwüste mit Rampe
    _hole(5, (20, 146), (80, 18),
          sand=[(8, 92, 84, 34)],
          slopes=[(20, 30, 60, 54, 0.0, 26.0)],
          walls=[(46, 128, 8, 22)],
          bumpers=[(20, 60, 6), (80, 60, 6)]),
    # 16 - Teichquerung über den Steg
    _hole(4, (50, 146), (50, 20),
          water=[(8, 56, 28, 52), (64, 56, 28, 52)],
          bumpers=[(24, 34, 6), (76, 34, 6)],
          sand=[(40, 116, 20, 14)]),
    # 17 - Kreuzmühle
    _hole(4, (18, 144), (82, 24),
          walls=[(6, 96, 26, 8), (68, 96, 26, 8)],
          mills=[(50, 100, 14, 3, 1.1)],
          sand=[(66, 118, 24, 18)]),
    # 18 - Finale: Tore, Wasser und Rampe
    _hole(5, (50, 148), (50, 18),
          water=[(8, 96, 30, 30), (62, 96, 30, 30)],
          walls=[(38, 92, 8, 6), (54, 92, 8, 6)],
          slopes=[(24, 40, 52, 44, 0.0, 30.0)],
          movers=[(24, 62, 22, 8, 30, 0, 24)],
          bumpers=[(50, 30, 7)]),
]


class MiniGolfGame(Game):
    name = "Minigolf"
    highscore_key = "minigolf"
    supports_multiplayer = True
    # Rechte Maustaste = Stärke-Sperre (siehe _lock_power).
    wants_right_click = True

    # ===================================================== Aufbau / Reset
    def reset(self):
        self.score = 0
        self.game_over = False
        gs = self.settings.get("minigolf", {}) if isinstance(self.settings, dict) else {}
        self.course = gs.get("course", "classic")
        if self.course not in COURSES:
            self.course = "classic"
        self.guide = bool(gs.get("guide", True))
        # Autoziel: der Schläger dreht sich vor jedem Schlag von selbst zum
        # Loch. Standard an, im Setup (oder mit Z) abschaltbar - dann bleibt
        # die zuletzt gewählte Richtung stehen und gezielt wird selbst.
        self.autoaim = bool(gs.get("autoaim", True))
        # Aufnehmen: nach MAX_STROKES Schlägen ist die Bahn vorbei. Standard an,
        # im Setup abschaltbar - dann wird bis zum Einlochen weitergespielt.
        self.pickup = bool(gs.get("pickup", True))
        # Stärke-Sperre: solange die rechte Maustaste gehalten wird, bleibt die
        # Schlagstärke stehen (lock_t treibt den Puls der Anzeige).
        self.power_lock = False
        self.lock_t = 0.0
        self.tour = max(1, min(gen.TOUR_COURSES, int(gs.get("tour", 1) or 1)))
        self._tour_par = gen.course_par(self.tour)
        self.winner = None
        # Größe der LAUFENDEN Bahn. Eingebaute und erzeugte Bahnen sind immer
        # CW x CH; eigene Bahnen aus dem Editor bringen ihre eigene mit.
        self.cw, self.ch = float(CW), float(CH)
        # Reiter des Vorbereitungs-Screens und der Bahn-Editor (siehe
        # minigolf_edit.py). Der Editor wird erst angelegt, wenn er gebraucht
        # wird - wer nie eigene Bahnen baut, zahlt dafür nichts.
        self.setup_tab = "play"
        self.maps = None
        self.editor = None
        # id der einzeln gespielten eigenen Bahn (leer = alle nacheinander).
        self.single_map = str(gs.get("ugc_map", "") or "")
        # Beim Test-Spielen aus dem Editor: hierhin geht es danach zurück.
        self._test_return = False
        # Rohr-Sperre und Schub-Sperre (siehe _physics).
        self._tun_cd = 0.0
        self._boost_cd = 0.0
        self.air = 0.0          # verbleibende Flugstrecke einer Sprungrampe
        self.air_dir = (0.0, 0.0)
        # Replay: Aufnahme der laufenden Runde (rec) und die fertige
        # Wiederholung der letzten Runde (replay, siehe replay.py).
        self.rec = None
        self.replay = None
        self.replay_request = None
        self._rep = None

        self._build_fonts()
        self._layout()
        self._build_setup_layout()
        self._over_cache = None
        self.best = self._load_best()
        self._new_round()
        self.state = SETUP

    def _build_fonts(self):
        h = self.height
        self._small = ui.font(max(14, h // 32))
        self._tiny = ui.font(max(11, h // 42))
        self._card = ui.font(max(11, h // 46), mono=True)
        self._huge = ui.font(max(24, h // 13), bold=True)

    def on_surface_changed(self):
        self._build_fonts()
        self._layout()
        self._build_setup_layout()
        self._over_cache = None
        # MAPS-Reiter und Editor rechnen ihr Layout selbst aus - sie müssen
        # nach einem Auflösungswechsel mitziehen.
        if self.maps is not None:
            self.maps.layout()
        if self.editor is not None:
            self.editor.layout()

    def _layout(self):
        """Maßstab und Nullpunkt des Platzes aus der Spielfläche ableiten.

        Gerechnet wird mit self.cw/self.ch, nicht mit CW/CH: eigene Bahnen
        aus dem Editor dürfen von 100x160 abweichen, und dann muss der Platz
        neu eingepasst werden (_start_hole ruft das deshalb je Bahn auf).
        """
        cw, ch = self.cw, self.ch
        self.hud_h = 44
        avail_h = self.height - self.hud_h - 12
        card_w = 132 if self.width >= 560 else 108
        self.scale = max(1.2, min((self.width - card_w - 40) / cw,
                                  avail_h / ch))
        self.ox = (self.width - card_w - 12) / 2.0 - cw * self.scale / 2.0
        self.oy = self.hud_h + (avail_h - ch * self.scale) / 2.0 + 6
        self.card_x = self.width - card_w - 6
        self.card_w = card_w
        self._build_over_layout()

    # ------------------------------------------------------- Speicherstand
    def _load_best(self):
        """Bestwerte je Kurs: {"classic": schläge, ...} (kleiner = besser)."""
        data = store.load_section("minigolf")
        best = data.get("best") if isinstance(data, dict) else None
        out = {}
        if isinstance(best, dict):
            for k, v in best.items():
                try:
                    out[str(k)] = int(v)
                except (TypeError, ValueError):
                    continue
        return out

    def _best_key(self):
        """Schlüssel des Bestwerts: je Tour-Kurs und je eigener Bahn einer."""
        if self.course == "tour":
            return "tour%d" % self.tour
        if self.course == "ugc" and self.single_map:
            return "ugc:" + self.single_map
        return self.course

    def _save_best(self, strokes):
        key = self._best_key()
        old = self.best.get(key)
        if old is None or strokes < old:
            self.best[key] = strokes
            store.save_section("minigolf", {"best": self.best})

    def _save_setting(self, key, value):
        if isinstance(self.settings, dict):
            self.settings.setdefault("minigolf", {})[key] = value
            settings_mod.save_settings(self.settings)

    # ------------------------------------------------------- Runde / Bahn
    def _round_holes(self):
        """Die 9 Bahnen der Runde (Random: gezogen und zufällig gespiegelt)."""
        if self.course == "classic":
            return [dict(h) for h in HOLES_CLASSIC]
        if self.course == "pro":
            return [dict(h) for h in HOLES_PRO]
        if self.course == "tour":
            return gen.course_holes(self.tour)
        if self.course == "ugc":
            return self._ugc_holes()
        # Random: aus handgebauten UND erzeugten Bahnen ziehen
        pool = list(HOLES_CLASSIC + HOLES_PRO)
        for _ in range(HOLES_PER_ROUND):
            pool.append(gen.generate(random.randint(1, gen.TOUR_COURSES),
                                     random.randrange(HOLES_PER_ROUND)))
        picked = random.sample(pool, HOLES_PER_ROUND)
        picked.sort(key=lambda h: h["par"])
        return [self._mirror(h, random.random() < 0.5) for h in picked]

    def _ugc_holes(self):
        """Die eigenen Bahnen als Runde.

        Eine einzelne Bahn wird gespielt, wenn sie aus dem MAPS-Reiter heraus
        gestartet wurde (self.single_map). Sonst laufen alle eigenen Bahnen
        der Reihe nach - die Runde ist dann so lang wie die Sammlung, aber
        höchstens neun Bahnen wie überall sonst.
        """
        import ugc
        if self.single_map:
            m = ugc.get(self.single_map)
            if m:
                return [ugc.to_hole(m)]
        maps = ugc.load_maps()
        if not maps:
            return [dict(HOLES_CLASSIC[0])]      # Notnagel: nie ohne Bahn
        return [ugc.to_hole(m) for m in maps[:HOLES_PER_ROUND]]

    @staticmethod
    def _mirror(hole, flip):
        """Spiegelt eine Bahn an der Mittelachse (für den Zufallskurs).

        Alles, was eine Richtung hat, dreht dabei sein x-Vorzeichen um -
        Rampen, Wanderblöcke, Schub, Tore und Sprungrampen. Mühlen und
        Drehscheiben laufen andersherum.
        """
        hole = gen.normalize(hole)
        if not flip:
            return dict(hole)
        cw = hole["w"]

        def mx(x, w=0.0):
            return cw - x - w

        out = dict(hole)
        out["tee"] = (mx(hole["tee"][0]), hole["tee"][1])
        out["cup"] = (mx(hole["cup"][0]), hole["cup"][1])
        for key in ("walls", "sand", "water", "ice", "sticky"):
            out[key] = [(mx(x, w), y, w, h) for (x, y, w, h) in hole[key]]
        out["slopes"] = [(mx(x, w), y, w, h, -ax, ay)
                         for (x, y, w, h, ax, ay) in hole["slopes"]]
        out["bumpers"] = [(mx(x), y, r) for (x, y, r) in hole["bumpers"]]
        out["movers"] = [(mx(x, w), y, w, h, -dx, dy, sp)
                         for (x, y, w, h, dx, dy, sp) in hole["movers"]]
        out["mills"] = [(mx(x), y, ln, arms, -sp)
                        for (x, y, ln, arms, sp) in hole["mills"]]
        out["tunnels"] = [(mx(x1), y1, mx(x2), y2, r)
                          for (x1, y1, x2, y2, r) in hole["tunnels"]]
        out["boosters"] = [(mx(x, w), y, w, h, -dx, dy, b)
                           for (x, y, w, h, dx, dy, b) in hole["boosters"]]
        out["magnets"] = [(mx(x), y, r, f) for (x, y, r, f) in hole["magnets"]]
        out["gates"] = [(mx(x, w), y, w, h, -dx, dy)
                        for (x, y, w, h, dx, dy) in hole["gates"]]
        out["spinners"] = [(mx(x), y, r, -sp)
                           for (x, y, r, sp) in hole["spinners"]]
        out["jumps"] = [(mx(x, w), y, w, h, -dx, dy, d)
                        for (x, y, w, h, dx, dy, d) in hole["jumps"]]
        return out

    def _new_round(self):
        self.holes = self._round_holes()
        self.players = 2 if self.multiplayer else 1
        self.cards = [[0] * len(self.holes) for _ in range(self.players)]
        self.points = [0] * self.players
        self.hole_idx = 0
        self.player = 0
        self.winner = None
        self.score = 0
        self.game_over = False
        self._rec_new()
        self._start_hole()

    def _start_hole(self):
        h = gen.normalize(self.holes[self.hole_idx])
        self.hole = h
        # Bahngröße übernehmen und den Platz neu einpassen - eigene Bahnen
        # dürfen kleiner oder größer als 100x160 sein.
        if (h["w"], h["h"]) != (self.cw, self.ch):
            self.cw, self.ch = h["w"], h["h"]
            self._layout()
        self.par = h["par"]
        self.strokes = 0
        self.cup = (float(h["cup"][0]), float(h["cup"][1]))
        self.bx, self.by = float(h["tee"][0]), float(h["tee"][1])
        self.vx = self.vy = 0.0
        self.safe = (self.bx, self.by)
        self.phase = "aim"
        if not self.power_lock:
            self.power = 0.35
        self.charging = False
        self.shot_time = 0.0
        self.mill_a = 0.0
        self.move_t = 0.0
        self._tun_cd = 0.0
        self._boost_cd = 0.0
        self.air = 0.0
        self.trail = []
        self.msg = None
        self.msg_t = 0.0
        self.result_key = None
        self.result_pts = 0
        self._layout_id = self.rec.layout(h) if self.rec else 0
        self._reset_aim(new_hole=True)

    # ===================================================== Setup-Screen
    def _build_setup_layout(self):
        """Vier Blöcke: Kurs, Tour-Kurs, Schalterzeile, Start.

        Die drei Schalter (Ziellinie, Autoziel, Aufnehmen) teilen sich eine
        Zeile - so bleiben auch bei 480x360 alle Optionen sichtbar, und die
        Knöpfe dürfen sogar höher ausfallen als in der alten Fünferliste.
        Was an Höhe übrig bleibt, geht in die Beschriftungsabstände.
        """
        cx = self.width // 2
        # Die Schriften wachsen mit der Höhe - der Block muss mitwachsen,
        # sonst passen lange Beschriftungen ("Ligne de visée") ab 800x600
        # nicht mehr über ihre Schaltergruppe. Bei 480x360 und 640x480 bleibt
        # es bei den bisherigen 370 px.
        bw = min(max(370, int(self.width * 0.58)), self.width - 50)
        gap = 8
        # Reiterzeile SPIEL | MAPS unter der Unterzeile. Darunter beginnt
        # beides: der gewohnte Kurs-Block und die Liste der eigenen Bahnen.
        # Der Kurs-Block rückt dafür um die Reiterhöhe nach unten; die 22 px
        # Abstand sind der Platz für die Beschriftung über der Kurszeile.
        tab_h = max(22, min(30, self.height // 15))
        tab_w = min(150, (self.width - 40) // 2)
        tab_y = int(self.height * 0.215)
        self.tab_rects = [
            pygame.Rect(cx - tab_w - 4, tab_y, tab_w, tab_h),
            pygame.Rect(cx + 4, tab_y, tab_w, tab_h)]
        self.tab_bottom = tab_y + tab_h
        # Der Abstand muss mit der Schrift wachsen: bei 1280x960 ist _tiny
        # 22 px hoch, dann reichen feste 22 px für die Beschriftung nicht mehr.
        top = self.tab_bottom + max(22, self._tiny.get_height() + 6)
        bottom = self.height - 42       # Platz für Bestwert- und Tastenzeile
        bh = max(26, min(42, int((bottom - top - 68) / 4)))
        # Was an Höhe übrig ist, kommt zur Hälfte auf die drei
        # Beschriftungsabstände - sonst klebt die Zeile bei 1280x960 in der
        # oberen Ecke, während darunter alles leer bleibt.
        lab = 20 + max(0, min(40, (bottom - top - 8 - 4 * bh - 60) // 6))
        step = bh + lab                 # lab px Platz für die Beschriftung

        def row(y, n, w=None, x0=None, h=None):
            """n gleich breite Felder nebeneinander (Standard: volle Breite)."""
            w = bw if w is None else w
            x0 = cx - bw / 2 if x0 is None else x0
            h = bh if h is None else h
            cw = (w - gap * (n - 1)) / n
            return [pygame.Rect(int(x0 + i * (cw + gap)), y, int(cw), h)
                    for i in range(n)]

        y = top
        self.course_rects = row(y, len(COURSES))
        y += step
        # Tour-Kurs: Pfeil links, Anzeige, Pfeil rechts
        self.tour_rects = [pygame.Rect(int(cx - bw / 2), y, 40, bh),
                           pygame.Rect(int(cx - bw / 2 + 46), y, int(bw - 92), bh),
                           pygame.Rect(int(cx + bw / 2 - 40), y, 40, bh)]
        y += step
        # Schalterzeile: drei AN/AUS-Paare nebeneinander. Die 18 px Luft
        # zwischen den Gruppen trennen deutlicher als die 8 px innerhalb einer.
        grp = (bw - 2 * 18) / 3.0
        self.opt_w = int(grp)
        self.guide_rects = row(y, 2, w=grp)
        self.autoaim_rects = row(y, 2, w=grp, x0=cx - grp / 2)
        self.pickup_rects = row(y, 2, w=grp, x0=cx + bw / 2 - grp)
        y += step + 4
        sw = max(190, int(bw * 0.42))
        self.start_rect = pygame.Rect(cx - sw // 2, y, sw, bh + 4)

    def _handle_setup(self, event):
        # Reiterwechsel geht in beiden Reitern zuerst.
        if event.kind == InputEvent.MOUSEDOWN:
            for i, rc in enumerate(self.tab_rects):
                if rc.collidepoint(event.pos):
                    self._set_tab(TABS[i])
                    return
        elif event.kind == InputEvent.KEYDOWN and event.key in ("Tab",
                                                                "ISO_Left_Tab"):
            self._set_tab("maps" if self.setup_tab == "play" else "play")
            return
        if self.setup_tab == "maps":
            if event.kind == InputEvent.KEYDOWN and event.key == "Escape":
                self._set_tab("play")
                return
            self.maps.handle(event)
            return
        if event.kind == InputEvent.KEYDOWN:
            k = event.key
            if k in ("1", "2", "3", "4", "5"):
                self.course = COURSES[int(k) - 1]
                self._save_setting("course", self.course)
                self.play_sound("click")
            elif k == "Left" or self.is_action(k, "left"):
                self._step_tour(-1)
            elif k == "Right" or self.is_action(k, "right"):
                self._step_tour(1)
            elif k in ("g", "G"):
                self._toggle_guide()
            elif k in ("z", "Z"):
                self._toggle_autoaim()
            elif k in ("p", "P"):
                self._toggle_pickup()
            elif k in ("Return", "space"):
                self._start_play()
        elif event.kind == InputEvent.MOUSEDOWN:
            for i, rc in enumerate(self.course_rects):
                if rc.collidepoint(event.pos):
                    self.course = COURSES[i]
                    self._save_setting("course", self.course)
                    self.play_sound("click")
                    return
            if self.course == "tour":
                if self.tour_rects[0].collidepoint(event.pos):
                    self._step_tour(-1)
                    return
                if self.tour_rects[2].collidepoint(event.pos):
                    self._step_tour(1)
                    return
            for rects, val, toggle in (
                    (self.guide_rects, self.guide, self._toggle_guide),
                    (self.autoaim_rects, self.autoaim, self._toggle_autoaim),
                    (self.pickup_rects, self.pickup, self._toggle_pickup)):
                for i, rc in enumerate(rects):
                    if rc.collidepoint(event.pos):
                        if val != (i == 0):
                            toggle()
                        return
            if self.start_rect.collidepoint(event.pos):
                self._start_play()

    def _step_tour(self, d):
        """Blättert durch die Tour-Kurse (nur wirksam, wenn Tour gewählt ist)."""
        if self.course != "tour":
            return
        self.tour = (self.tour - 1 + d) % gen.TOUR_COURSES + 1
        self._tour_par = gen.course_par(self.tour)
        self._save_setting("tour", self.tour)
        self.play_sound("move")

    def _toggle_guide(self):
        self.guide = not self.guide
        self._save_setting("guide", self.guide)
        self.play_sound("select")

    def _toggle_autoaim(self):
        self.autoaim = not self.autoaim
        self._save_setting("autoaim", self.autoaim)
        self.play_sound("select")

    def _toggle_pickup(self):
        self.pickup = not self.pickup
        self._save_setting("pickup", self.pickup)
        self.play_sound("select")

    def _start_play(self, single=None):
        """Runde starten.

        'single' ist die id genau einer eigenen Bahn (aus dem MAPS-Reiter).
        Ohne sie spielt "Eigene" die ganze Sammlung nacheinander - der
        START-Knopf im Setup ruft deshalb ohne Argument auf.
        """
        if self.course == "ugc":
            self.single_map = single or ""
        self._new_round()
        self.state = PLAY
        self.play_sound("click")

    # ===================================================== Eigene Bahnen
    # Der MAPS-Reiter und der Editor stecken in minigolf_edit.py; hier stehen
    # nur die Übergänge, die beide brauchen.

    @property
    def wants_escape(self):
        """ESC heißt im Editor und im MAPS-Reiter "Abbrechen", nicht "Pause".

        Auch beim Test-Spielen aus dem Editor: dort bricht ESC den Versuch ab
        und führt zurück ans Bauen.
        """
        return (self.state == EDIT or self._test_return
                or (self.state == SETUP and self.setup_tab == "maps"))

    @property
    def grid_snap(self):
        gs = self.settings.get("minigolf", {}) if isinstance(self.settings,
                                                             dict) else {}
        return bool(gs.get("grid", True))

    def set_grid_snap(self, on):
        self._save_setting("grid", bool(on))

    def _set_tab(self, tab):
        if tab not in TABS or tab == self.setup_tab:
            return
        self.setup_tab = tab
        if tab == "maps":
            from . import minigolf_edit as edit
            if self.maps is None:
                self.maps = edit.MapList(self)
            else:
                self.maps.layout()
                self.maps.reload()
        self.play_sound("click")

    def ugc_new_map(self):
        """Neue eigene Bahn anlegen und gleich in den Editor springen."""
        import ugc
        self.ugc_edit(ugc.new_map(author=ugc.last_author()))

    def ugc_edit(self, m):
        from . import minigolf_edit as edit
        self.editor = edit.MapEditor(self, m)
        self.state = EDIT
        self.play_sound("click")

    def ugc_close_editor(self):
        """Zurück aus dem Editor in den MAPS-Reiter."""
        self.editor = None
        self.state = SETUP
        self.setup_tab = "maps"
        if self.maps is not None:
            self.maps.reload()
        self.play_sound("click")

    def ugc_play(self, map_id):
        """Genau eine eigene Bahn spielen (aus dem MAPS-Reiter)."""
        self.course = "ugc"
        self._save_setting("course", "ugc")
        self._save_setting("ugc_map", map_id)
        self.best = self._load_best()
        self._start_play(single=map_id)

    def ugc_test(self, hole):
        """Bahn aus dem Editor sofort ausprobieren (danach zurück in den Editor)."""
        self._test_return = True
        self.course = "ugc"
        self.holes = [gen.normalize(hole)]
        self.players = 1
        self.cards = [[0] * len(self.holes)]
        self.points = [0]
        self.hole_idx = 0
        self.player = 0
        self.winner = None
        self.score = 0
        self.game_over = False
        self.rec = None                 # Testläufe werden nicht aufgezeichnet
        self.replay = None
        self._start_hole()
        self.state = PLAY
        self.play_sound("click")

    def _back_to_editor(self):
        """Nach dem Test-Spielen zurück in den Editor."""
        self._test_return = False
        self.state = EDIT
        self.game_over = False
        if self.editor is None:
            self.ugc_close_editor()

    # ===================================================== Eingabe
    def handle_event(self, event):
        # Die rechte Maustaste ist die Stärke-Sperre - und zwar in jedem
        # Zustand zuerst, damit sie nirgends als Linksklick durchrutscht
        # (der Setup-Screen fragt die Maustaste selbst nicht ab).
        if event.kind in (InputEvent.MOUSEDOWN, InputEvent.MOUSEUP) \
                and event.button == 3:
            self._lock_power(event.kind == InputEvent.MOUSEDOWN)
            return
        # Der Bahn-Editor hat eine eigene, vollständige Bedienung - solange er
        # offen ist, bekommt er alles (auch G/Z/P/F, die dort andere Bedeutung
        # haben, und ESC zum Abbrechen statt Pause, siehe wants_escape).
        if self.state == EDIT and self.editor is not None:
            self.editor.handle(event)
            return
        # Test-Spielen: ESC bricht ab und führt zurück in den Editor.
        if (self._test_return and event.kind == InputEvent.KEYDOWN
                and event.key == "Escape"):
            self._back_to_editor()
            return
        if self.state == SETUP:
            self._handle_setup(event)
            return
        if event.kind == InputEvent.KEYDOWN and event.key in ("g", "G"):
            self._toggle_guide()
            return
        if event.kind == InputEvent.KEYDOWN and event.key in ("z", "Z"):
            self._toggle_autoaim()
            return
        if event.kind == InputEvent.KEYDOWN and event.key in ("p", "P"):
            if self.state == OVER and self.replay is not None:
                self._open_replay()
            else:
                self._toggle_pickup()
            return
        if event.kind == InputEvent.KEYDOWN and event.key in ("f", "F"):
            self._reset_hole()
            return
        if self.state == HOLE_DONE:
            if (event.kind == InputEvent.KEYDOWN
                    and event.key in ("Return", "space")) \
                    or (event.kind == InputEvent.MOUSEDOWN and event.button == 1):
                self._advance()
            return
        if self.state == OVER:
            # Knöpfe: Weiter (nächster Kurs) · Nochmal · Setup.
            if event.kind == InputEvent.KEYDOWN:
                if event.key in ("Return", "space"):
                    self._over_action("next" if self._next_course() else "again")
                elif event.key in ("r", "R"):
                    self._over_action("again")
                elif event.key in ("s", "S"):
                    self._over_action("setup")
            elif event.kind == InputEvent.MOUSEDOWN and event.button == 1:
                for key, rc in self.over_rects:
                    if rc.collidepoint(event.pos):
                        self._over_action(key)
                        break
            return
        if self.state != PLAY or self.phase != "aim":
            return
        if event.kind == InputEvent.MOUSEMOVE:
            mx, my = self._unproject(*event.pos)
            if abs(mx - self.bx) > 0.4 or abs(my - self.by) > 0.4:
                self.aim = math.atan2(my - self.by, mx - self.bx)
        elif event.kind == InputEvent.MOUSEDOWN and event.button == 1:
            self.charging = True
            if not self.power_lock:      # gesperrt = mit dem Wert schlagen
                self.power = 0.05
        elif event.kind == InputEvent.MOUSEUP and event.button == 1:
            if self.charging:
                self.charging = False
                self._strike()
        elif event.kind == InputEvent.KEYDOWN:
            k = event.key
            if k == "Left" or self.is_action(k, "left"):
                self.aim -= math.radians(2.5)
            elif k == "Right" or self.is_action(k, "right"):
                self.aim += math.radians(2.5)
            elif k == "Up" or self.is_action(k, "up"):
                if not self.power_lock:
                    self.power = min(1.0, self.power + 0.05)
            elif k == "Down" or self.is_action(k, "down"):
                if not self.power_lock:
                    self.power = max(0.05, self.power - 0.05)
            elif k in ("r", "R"):
                self._cancel_shot()
            elif k in ("space", "Return"):
                self._strike()

    def _lock_power(self, on):
        """Stärke-Sperre: die rechte Maustaste hält die Schlagstärke fest.

        Gedrückt halten friert den Ladebalken genau da ein, wo er beim Drücken
        stand - Loslassen lädt weiter. So lässt sich ein fertig geladener
        Schlag beliebig lange halten und genau dann putten, wenn die Lücke in
        der Windmühle passt oder der Wanderblock aus dem Weg ist.

        Gesperrt bleibt die Stärke, bis die Taste losgelassen wird: sie
        übersteht Schlag, Bahnwechsel und [R], und ein Linksklick lädt dann
        nicht bei 5% neu, sondern schlägt exakt mit dem gehaltenen Wert. Auch
        Pfeil hoch/runter ändern währenddessen nichts. Gesperrt wird nur beim
        Zielen; freigegeben immer, damit die Sperre nie hängen bleibt.
        """
        if on and (self.state != PLAY or self.phase != "aim"):
            return
        if on == self.power_lock:
            return
        self.power_lock = on
        self.lock_t = 0.0
        self.play_sound("select" if on else "click")

    def _cancel_shot(self):
        """Bricht einen geladenen Schlag ab (Taste R).

        Wer die Maustaste hält und es sich anders überlegt, drückt R: der Ball
        bleibt liegen, der Schlag zählt nicht. Nach dem Loslassen lässt sich
        ganz normal neu aufladen. Eine gehaltene Stärke-Sperre bleibt dabei
        stehen - abgebrochen wird der Schlag, nicht der gemerkte Wert.
        """
        if not self.charging:
            return
        self.charging = False
        if not self.power_lock:
            self.power = 0.35
        self.msg = t("golf.cancel")
        self.msg_t = 1.4
        self.play_sound("click")

    def _strike(self):
        sp = MAX_SPEED * self.power
        self.vx = math.cos(self.aim) * sp
        self.vy = math.sin(self.aim) * sp
        self.strokes += 1
        self.phase = "rolling"
        self.shot_time = 0.0
        self.safe = (self.bx, self.by)
        self.trail = []
        if self.rec:
            self.rec.scene(flat=True, layout=self._layout_id,
                           hole=self.hole_idx, player=self.player,
                           n=self.strokes, aim=round(self.aim, 4),
                           power=round(self.power, 3),
                           mill=round(self.mill_a, 3),
                           move=round(self.move_t, 3))
        self.play_sound("shoot")
        self.rumble(50)

    # ===================================================== Projektion
    def _view(self):
        """Nullpunkt + Maßstab als View für minigolf_draw."""
        return draw.View(self.ox, self.oy, self.scale)

    def _project(self, x, y):
        return (self.ox + x * self.scale, self.oy + y * self.scale)

    def _unproject(self, sx, sy):
        return ((sx - self.ox) / self.scale, (sy - self.oy) / self.scale)

    def _rect_px(self, r):
        px, py = self._project(r[0], r[1])
        return pygame.Rect(int(px), int(py), max(1, int(r[2] * self.scale)),
                           max(1, int(r[3] * self.scale)))

    # ===================================================== Update / Physik
    def update(self, dt):
        if self.msg_t > 0:
            self.msg_t -= dt
            if self.msg_t <= 0:
                self.msg = None
        if self.state == EDIT:
            if self.editor is not None:
                self.editor.update(dt)
            return
        if self.state == SETUP and self.setup_tab == "maps" and self.maps:
            self.maps.update(dt)
            return
        if self.state != PLAY:
            return
        self.mill_a += dt
        self.move_t += dt
        if self.phase == "aim":
            if self.power_lock:
                self.lock_t += dt            # Puls der goldenen Anzeige
            elif self.charging:
                self.power = min(1.0, self.power + dt * 0.80)
        elif self.phase == "rolling":
            self._physics(dt)
            if self.rec:
                self.rec.tick(dt, self._rec_sample)
            self.shot_time += dt
            if self.shot_time > MAX_SHOT_TIME:
                self.vx = self.vy = 0.0
                self.air = 0.0        # notfalls auch aus dem Flug holen
            if self.phase == "rolling" and self.vx == 0.0 and self.vy == 0.0:
                self._after_shot()

    def _mover_rect(self, m):
        """Aktuelles Rechteck eines Wanderblocks (pendelt zwischen den Enden)."""
        return draw.mover_rect(m, self.move_t)

    def _physics(self, dt):
        speed = math.hypot(self.vx, self.vy)
        steps = max(2, min(24, int(speed * dt / BR) + 2))
        h = dt / steps
        for _ in range(steps):
            self._tun_cd = max(0.0, self._tun_cd - h)
            self._boost_cd = max(0.0, self._boost_cd - h)
            # --- Sprungrampe: solange der Ball fliegt, gibt es nur die Bande.
            # Hindernisse, Loch und Wasser werden überflogen - genau dafür ist
            # die Schanze da.
            if self.air > 0.0:
                move = math.hypot(self.vx, self.vy) * h
                self.air -= move
                self.bx += self.vx * h
                self.by += self.vy * h
                self._collide_bounds()
                # Ohne Tempo gibt es keinen Flug mehr - sonst bliebe der Ball
                # ewig in der Luft und die Bahn liefe nie zu Ende.
                if self.air <= 0.0 or move <= 0.0:
                    self.air = 0.0
                    self.play_sound("hit")
                continue
            fr = {"sand": FRIC_SAND, "ice": FRIC_ICE,
                  "sticky": FRIC_STICKY}.get(self._terrain(), FRIC_GREEN)
            # Rampen beschleunigen, solange der Ball darauf liegt
            for (x, y, w, hh, ax, ay) in self.hole["slopes"]:
                if x <= self.bx <= x + w and y <= self.by <= y + hh:
                    self.vx += ax * h
                    self.vy += ay * h
            self._apply_magnets(h)
            f = max(0.0, 1.0 - fr * h)
            self.vx *= f
            self.vy *= f
            self.bx += self.vx * h
            self.by += self.vy * h
            self._collide_bounds()
            for r in self.hole["walls"]:
                self._collide_rect(r)
            for g in self.hole["gates"]:
                self._collide_gate(g)
            for m in self.hole["movers"]:
                rect, vel = self._mover_rect(m)
                self._collide_rect(rect, vel)
            for b in self.hole["bumpers"]:
                self._collide_bumper(b)
            for mill in self.hole["mills"]:
                self._collide_mill(mill)
            for sp in self.hole["spinners"]:
                self._collide_spinner(sp, h)
            self._apply_boosters()
            if self._check_jumps() or self._check_tunnels():
                continue
            if self._check_cup() or self._check_water():
                return
            if self.vx * self.vx + self.vy * self.vy < STOP_EPS * STOP_EPS:
                self.vx = self.vy = 0.0
                return
        self.trail.append((self.bx, self.by))
        if len(self.trail) > 26:
            del self.trail[0]

    def _terrain(self):
        """Untergrund unter dem Ball: green, sand, ice oder sticky.

        Bei Überlappung gewinnt der bremsendste - so bleibt eine Sandinsel
        auf einer Eisfläche auch wirklich Sand.
        """
        for key in ("sticky", "sand", "ice"):
            for (x, y, w, h) in self.hole[key]:
                if x <= self.bx <= x + w and y <= self.by <= y + h:
                    return key
        return "green"

    # ----- Die acht Editor-Hindernisse ---------------------------------
    def _apply_magnets(self, h):
        """Magnet: zieht den Ball an (force > 0) oder stößt ihn ab."""
        for (x, y, r, force) in self.hole["magnets"]:
            dx, dy = x - self.bx, y - self.by
            d = math.hypot(dx, dy)
            if d >= r or d < 0.4:
                continue
            # Nah an der Mitte stärker, am Rand des Feldes gar nicht.
            pull = force * (1.0 - d / r) * h / d
            self.vx += dx * pull
            self.vy += dy * pull

    def _collide_gate(self, g):
        """Einbahn-Tor: in Richtung (dx, dy) offen, dagegen eine Wand."""
        x, y, w, h, dx, dy = g
        if self.vx * dx + self.vy * dy > 0:
            return                     # richtige Richtung -> freie Fahrt
        self._collide_rect((x, y, w, h))

    def _collide_spinner(self, sp, h):
        """Drehscheibe: nimmt den Ball mit und trägt ihn nach außen."""
        x, y, r, speed = sp
        dx, dy = self.bx - x, self.by - y
        d = math.hypot(dx, dy)
        if d >= r:
            return
        # Umfangsgeschwindigkeit am Ort des Balls + leichte Fliehkraft
        self.vx += (-dy * speed - self.vx) * SPIN_PUSH * h
        self.vy += (dx * speed - self.vy) * SPIN_PUSH * h
        if d > 0.4:
            self.vx += dx / d * abs(speed) * 4.0 * h
            self.vy += dy / d * abs(speed) * 4.0 * h

    def _apply_boosters(self):
        """Schub-Feld: einmaliger Stoß beim Betreten (nicht je Teilschritt)."""
        if self._boost_cd > 0.0:
            return
        for (x, y, w, h, dx, dy, boost) in self.hole["boosters"]:
            if not (x <= self.bx <= x + w and y <= self.by <= y + h):
                continue
            n = math.hypot(dx, dy)
            if n < 1e-6 or boost <= 0:
                continue
            ux, uy = dx / n, dy / n
            # Auf mindestens 'boost' beschleunigen - schneller wird nie gebremst.
            along = self.vx * ux + self.vy * uy
            if along < boost:
                self.vx += ux * (boost - along)
                self.vy += uy * (boost - along)
                self._boost_cd = BOOST_COOLDOWN
                self.play_sound("bounce")
            return

    def _check_jumps(self):
        """Sprungrampe: hebt den Ball ab, wenn er schnell genug drüberrollt."""
        if self.air > 0.0:
            return False
        for (x, y, w, h, dx, dy, dist) in self.hole["jumps"]:
            if not (x <= self.bx <= x + w and y <= self.by <= y + h):
                continue
            sp = math.hypot(self.vx, self.vy)
            if sp < JUMP_MIN_SPEED:
                return False           # zu langsam: die Schanze tut nichts
            n = math.hypot(dx, dy)
            if n > 1e-6:               # in Sprungrichtung ausrichten
                self.vx, self.vy = dx / n * sp, dy / n * sp
            self.air = max(1.0, float(dist))
            self.play_sound("bounce")
            return True
        return False

    def _check_tunnels(self):
        """Rohr: versetzt den Ball ans andere Ende, Richtung bleibt erhalten."""
        if self._tun_cd > 0.0:
            return False
        for (x1, y1, x2, y2, r) in self.hole["tunnels"]:
            for (ex, ey, ox, oy) in ((x1, y1, x2, y2), (x2, y2, x1, y1)):
                if math.hypot(self.bx - ex, self.by - ey) > r:
                    continue
                sp = math.hypot(self.vx, self.vy) * TUNNEL_KEEP
                if sp < 1e-6:
                    return False       # liegen geblieben: kein Transport
                ux, uy = self.vx / math.hypot(self.vx, self.vy), \
                    self.vy / math.hypot(self.vx, self.vy)
                # Direkt hinter der Mündung wieder ausspucken, sonst fängt
                # das Ziel-Ende den Ball sofort wieder ein.
                self.bx = ox + ux * (r + BR + 0.5)
                self.by = oy + uy * (r + BR + 0.5)
                self.vx, self.vy = ux * sp, uy * sp
                self._tun_cd = TUNNEL_COOLDOWN
                self.play_sound("hit")
                return True
        return False

    def _collide_bounds(self):
        lo = BORDER + BR
        hix, hiy = self.cw - BORDER - BR, self.ch - BORDER - BR
        if self.bx < lo:
            self.bx, self.vx = lo, abs(self.vx) * WALL_E
            self._thud()
        elif self.bx > hix:
            self.bx, self.vx = hix, -abs(self.vx) * WALL_E
            self._thud()
        if self.by < lo:
            self.by, self.vy = lo, abs(self.vy) * WALL_E
            self._thud()
        elif self.by > hiy:
            self.by, self.vy = hiy, -abs(self.vy) * WALL_E
            self._thud()

    def _thud(self):
        if abs(self.vx) + abs(self.vy) > 45:
            self.play_sound("bounce")

    def _collide_rect(self, r, vel=(0.0, 0.0)):
        """Kreis gegen Rechteck: nächster Punkt, herausschieben, reflektieren."""
        x, y, w, h = r[0], r[1], r[2], r[3]
        nx = max(x, min(self.bx, x + w))
        ny = max(y, min(self.by, y + h))
        dx, dy = self.bx - nx, self.by - ny
        d2 = dx * dx + dy * dy
        if d2 >= BR * BR:
            return
        if d2 > 1e-9:
            d = math.sqrt(d2)
            ux, uy, push = dx / d, dy / d, BR - d
        else:
            # Mittelpunkt im Rechteck -> über die kürzeste Achse hinausschieben
            left, right = self.bx - x, x + w - self.bx
            top, bottom = self.by - y, y + h - self.by
            m = min(left, right, top, bottom)
            if m == left:
                ux, uy, push = -1.0, 0.0, left + BR
            elif m == right:
                ux, uy, push = 1.0, 0.0, right + BR
            elif m == top:
                ux, uy, push = 0.0, -1.0, top + BR
            else:
                ux, uy, push = 0.0, 1.0, bottom + BR
        self.bx += ux * push
        self.by += uy * push
        rvx, rvy = self.vx - vel[0], self.vy - vel[1]
        dot = rvx * ux + rvy * uy
        if dot < 0:
            rvx -= (1 + WALL_E) * dot * ux
            rvy -= (1 + WALL_E) * dot * uy
            self.vx = rvx + vel[0] * 1.4
            self.vy = rvy + vel[1] * 1.4
            self._thud()

    def _collide_bumper(self, b):
        x, y, r = b
        dx, dy = self.bx - x, self.by - y
        d = math.hypot(dx, dy)
        rad = r + BR
        if d >= rad or d < 1e-9:
            return
        ux, uy = dx / d, dy / d
        self.bx, self.by = x + ux * rad, y + uy * rad
        dot = self.vx * ux + self.vy * uy
        if dot < 0:
            self.vx -= (1 + BUMP_E) * dot * ux
            self.vy -= (1 + BUMP_E) * dot * uy
            self.play_sound("bounce")

    def _collide_mill(self, mill):
        x, y, length, arms, speed = mill
        base = self.mill_a * speed
        for i in range(int(arms)):
            a = base + i * (2 * math.pi / int(arms))
            ex, ey = x + math.cos(a) * length, y + math.sin(a) * length
            px, py = self._closest_on_seg(x, y, ex, ey, self.bx, self.by)
            dx, dy = self.bx - px, self.by - py
            d = math.hypot(dx, dy)
            rad = BR + ARM_W
            if d >= rad:
                continue
            if d < 1e-9:
                dx, dy, d = 0.0, -1.0, 1.0
            ux, uy = dx / d, dy / d
            self.bx, self.by = px + ux * rad, py + uy * rad
            # Umfangsgeschwindigkeit des Flügels am Kontaktpunkt
            rvx, rvy = -(py - y) * speed, (px - x) * speed
            relx, rely = self.vx - rvx, self.vy - rvy
            dot = relx * ux + rely * uy
            if dot < 0:
                relx -= 1.8 * dot * ux
                rely -= 1.8 * dot * uy
            self.vx, self.vy = relx + rvx, rely + rvy
            self.play_sound("hit")
            return

    @staticmethod
    def _closest_on_seg(x1, y1, x2, y2, px, py):
        dx, dy = x2 - x1, y2 - y1
        l2 = dx * dx + dy * dy
        if l2 < 1e-9:
            return x1, y1
        f = max(0.0, min(1.0, ((px - x1) * dx + (py - y1) * dy) / l2))
        return x1 + f * dx, y1 + f * dy

    def _check_cup(self):
        dx, dy = self.cup[0] - self.bx, self.cup[1] - self.by
        d = math.hypot(dx, dy)
        sp = math.hypot(self.vx, self.vy)
        if d < CUP_R + BR and sp < CAPTURE_SPEED * 1.6:
            # Sog Richtung Loch - fühlt sich an wie eine echte Lochkante
            pull = 26.0 * (1.0 - d / (CUP_R + BR)) / max(d, 0.4)
            self.vx += dx * pull
            self.vy += dy * pull
        if d < CUP_R * 0.75 and sp < CAPTURE_SPEED:
            self.vx = self.vy = 0.0
            self.bx, self.by = self.cup
            self._holed()
            return True
        return False

    def _check_water(self):
        for (x, y, w, h) in self.hole["water"]:
            if x <= self.bx <= x + w and y <= self.by <= y + h:
                self.vx = self.vy = 0.0
                self.strokes += 1
                self._rec_end("water")
                self.bx, self.by = self.safe
                self.msg = t("golf.penalty")
                self.msg_t = 2.0
                self.play_sound("hit")
                if self.pickup and self.strokes >= MAX_STROKES:
                    self._finish_hole(holed=False)
                else:
                    self.phase = "aim"
                    self._reset_aim()
                return True
        return False

    def _reset_aim(self, new_hole=False):
        """Zielrichtung und Kraft für den nächsten Schlag setzen.

        Mit Autoziel (Standard) zeigt der Schläger vor jedem Schlag zum Loch.
        Ohne Autoziel bleibt die zuletzt gewählte Richtung stehen - gezielt
        wird selbst. Nur am Tee einer neuen Bahn gibt es keine "letzte"
        Richtung; dort zeigt der Schläger neutral bahnaufwärts (nach oben),
        damit niemand mit dem Rücken zur Bahn startet.

        Eine gehaltene Stärke-Sperre überlebt den Schlag: dann bleibt auch die
        Kraft stehen, statt auf den Standardwert zurückzufallen.
        """
        if self.autoaim:
            self.aim = math.atan2(self.cup[1] - self.by, self.cup[0] - self.bx)
        elif new_hole:
            self.aim = -math.pi / 2
        if not self.power_lock:
            self.power = 0.35
        self.charging = False

    def _after_shot(self):
        self._rec_end("stop")
        if self.pickup and self.strokes >= MAX_STROKES:
            self.msg = t("golf.max_strokes")
            self.msg_t = 2.4
            self._finish_hole(holed=False)
            return
        self.phase = "aim"
        self._reset_aim()

    # ===================================================== Bahn abschließen
    def _holed(self):
        self._rec_end("cup")
        self.play_sound("win" if self.strokes == 1 else "point")
        self.rumble(90)
        px, py = self._project(*self.cup)
        ui.spawn_burst(px, py, self.accent, n=16)
        self._finish_hole(holed=True)

    def _finish_hole(self, holed):
        self.cards[self.player][self.hole_idx] = self.strokes
        if holed:
            pts = max(100, 600 + (self.par - self.strokes) * 300)
            if self.strokes == 1:
                pts += 500
                self.ach_event("golf_ace")
        else:
            pts = 100
        self.points[self.player] += pts
        self.result_pts = pts
        self.result_key = self._result_key(self.strokes, self.par, holed)
        if self.rec:
            self.rec.set_last(final=self.strokes, result=self.result_key,
                              pts=pts, holed=bool(holed))
        if self.player == 0:
            self.score = self.points[0]
        self.phase = "done"
        self.state = HOLE_DONE

    @staticmethod
    def _result_key(strokes, par, holed):
        if not holed:
            return "golf.res.max"
        if strokes == 1:
            return "golf.res.ace"
        d = strokes - par
        if d <= -2:
            return "golf.res.eagle"
        if d == -1:
            return "golf.res.birdie"
        if d == 0:
            return "golf.res.par"
        if d == 1:
            return "golf.res.bogey"
        if d == 2:
            return "golf.res.double"
        return "golf.res.over"

    def _advance(self):
        """Nächster Spieler bzw. nächste Bahn (oder Rundenende)."""
        self.play_sound("click")
        if self.player + 1 < self.players:
            self.player += 1
            self.state = PLAY
            self._start_hole()
            return
        self.player = 0
        if self.hole_idx + 1 < len(self.holes):
            self.hole_idx += 1
            self.state = PLAY
            self._start_hole()
            return
        self._end_round()

    # ------------------------------------------------- Rundenende / Weiter
    def _next_course(self):
        """(Kurs, Tour-Nummer) des nächsten Kurses - oder None.

        Reihenfolge: Classic -> Pro -> Tour 1 -> Tour 2 -> ... -> Tour 38.
        So bekommt man am Rundenende nie wieder denselben Neuner-Satz
        vorgesetzt. Random zieht ohnehin jedes Mal neue Bahnen und hat
        deshalb kein Ziel (dort ist "Nochmal" bereits eine neue Runde).
        """
        if self.course == "classic":
            return ("pro", self.tour)
        if self.course == "pro":
            return ("tour", 1)
        if self.course == "tour" and self.tour < gen.TOUR_COURSES:
            return ("tour", self.tour + 1)
        return None

    def _next_course_label(self):
        """Name des nächsten Kurses für die Knopfbeschriftung."""
        nxt = self._next_course()
        if nxt is None:
            return ""
        course, num = nxt
        name = t("golf.course." + course)
        return "%s %d" % (name, num) if course == "tour" else name

    def _over_keys(self):
        """Knöpfe des Rundenende-Bildschirms (nur die Schlüssel)."""
        keys = ["again", "setup"]
        if getattr(self, "replay", None) is not None:
            keys.insert(0, "replay")
        if self._next_course() is not None:
            keys.insert(0, "next")
        return keys

    def _build_over_layout(self):
        """Banner-Höhe, Zeilen-Positionen und Knopfreihe des Rundenendes.

        Alles wächst mit den Schriftgrößen mit, damit der Titel bei 1280x960
        genauso sauber sitzt wie bei 480x360. Gespeichert werden die
        Mitten-Abstände ab Banner-Oberkante (self.over_y) und die Knöpfe
        als (Schlüssel, Rect)-Paare.
        """
        keys = self._over_keys()
        head_h = self._huge.get_height()
        line_h = self._small.get_height()
        tiny_h = self._tiny.get_height()
        bh = max(26, min(38, line_h + 10))

        y = OVER_PAD
        self.over_y = {}
        for name, h, gap in (("head", head_h, 6), ("sub", line_h, 4),
                             ("best", tiny_h, 10)):
            self.over_y[name] = y + h // 2
            y += h + gap
        btn_top = y
        y += bh + 8
        self.over_y["hint"] = y + tiny_h // 2
        self.over_h = y + tiny_h + OVER_PAD

        gap = 8
        bw = min(int(self.width * 0.74), 660)
        cw = (bw - gap * (len(keys) - 1)) / len(keys)
        cx = self.width // 2
        top = self.height // 2 - self.over_h // 2 + btn_top
        self.over_rects = [
            (key, pygame.Rect(int(cx - bw / 2 + i * (cw + gap)), top,
                              int(cw), bh))
            for i, key in enumerate(keys)]

    def _continue_next(self):
        """Weiter-Knopf: nächsten Kurs laden und sofort abschlagen."""
        nxt = self._next_course()
        if nxt is None:
            self._restart()
            return
        self.course, self.tour = nxt
        self._tour_par = gen.course_par(self.tour)
        self._save_setting("course", self.course)
        self._save_setting("tour", self.tour)
        self._build_over_layout()
        self._new_round()
        self.state = PLAY
        self.play_sound("click")

    def _over_action(self, key):
        """Führt einen Knopf des Rundenende-Bildschirms aus."""
        if key == "replay":
            self._open_replay()
        elif key == "next" and self._next_course() is not None:
            self._continue_next()
        elif key == "setup":
            self.state = SETUP
            self.game_over = False
            self.play_sound("click")
        else:
            self._restart()

    def _reset_hole(self):
        """Taste F: die laufende Bahn von vorn.

        Schläge zurück auf 0, Ball zurück aufs Tee - gleiche Bahn, gleicher
        Spieler, gleicher Kurs. Bereits abgeschlossene Bahnen der Runde
        bleiben in der Scorekarte stehen.
        """
        if self.state != PLAY:
            return
        if self.rec:
            self.rec.drop_where(hole=self.hole_idx, player=self.player)
        self._start_hole()
        self.msg = t("golf.reset")
        self.msg_t = 1.4
        self.play_sound("click")

    def _end_round(self):
        # Test-Spielen aus dem Editor: kein Bestwert, kein Erfolg, keine
        # Statistik - es geht direkt zurück ans Bauen.
        if self._test_return:
            self._back_to_editor()
            return
        total = sum(self.cards[0])
        par_total = sum(h["par"] for h in self.holes)
        self._save_best(total)
        if total < par_total:
            self.ach_event("golf_under_par")
        if self.multiplayer:
            t0, t1 = sum(self.cards[0]), sum(self.cards[1])
            self.winner = 0 if t0 < t1 else (1 if t1 < t0 else None)
        else:
            self.winner = None
            self.report_result(total <= par_total)
        self.score = self.points[0]
        self._rec_finish(total, par_total)
        self._build_over_layout()
        self.state = OVER
        self.game_over = True
        self.play_sound("win")

    def _restart(self):
        self._new_round()
        self.state = PLAY
        self.play_sound("click")

    # ===================================================== Replay-Aufnahme
    # Aufgezeichnet wird je Schlag die tatsächlich gerollte Ballbahn (siehe
    # replay.py). Der Rest der Bahn - Banden, Sand, Wasser, Mühlen - liegt
    # als Kulisse im Replay, damit die Wiederholung auch dann noch stimmt,
    # wenn spätere Versionen die Bahnen ändern.

    def _rec_new(self):
        """Startet die Aufzeichnung der Runde (falls Replays an sind)."""
        self.replay = None
        self.rec = replay.recorder("minigolf", self.settings, meta={
            "course": self.course, "tour": self.tour,
            "players": self.players})

    def _rec_sample(self):
        """Ein Sample = die Ballposition (auf 1/100 Feldeinheit gerundet)."""
        return (round(self.bx, 2), round(self.by, 2))

    def _rec_end(self, end):
        """Beendet die laufende Schlag-Sequenz ("cup"/"water"/"stop")."""
        if self.rec:
            self.rec.close(self._rec_sample, end=end, after=self.strokes)

    def _rec_finish(self, total, par_total):
        """Rundenende: die Aufnahme als self.replay bereitlegen."""
        if not self.rec:
            return
        d = total - par_total
        name = t("golf.course." + self.course)
        if self.course == "tour":
            name = "%s %d" % (name, self.tour)
        self.replay = self.rec.result(
            title=name,
            sub=t("golf.final", strokes=total,
                  diff=("%+d" % d) if d else t("golf.even"),
                  pts=self.points[0]),
            total=total, par=par_total, points=self.points[0])
        self.rec = None

    def _open_replay(self):
        """Rundenende: die Wiederholung ansehen (Taste P bzw. Knopf).

        Den Screen öffnet main.py - das Spiel legt nur den Wunsch ab.
        """
        if self.replay is not None:
            self.replay_request = self.replay
            self.play_sound("click")

    # ===================================================== Replay-Wiedergabe
    # Der Replay-Screen (replayview.py) baut eine ganz normale Spielinstanz
    # und fährt sie über diese drei Methoden Bild für Bild durch die
    # Aufnahme - so zeichnet die Wiederholung mit demselben Code wie das
    # Spiel selbst.

    def replay_begin(self, rep):
        """Schaltet diese Instanz auf reine Wiedergabe um."""
        self.rec = None
        self.replay = None
        self.replay_request = None
        self._rep = rep
        # Aufnahmen aus älteren Versionen kennen weder die Bahngröße noch die
        # neuen Hindernis-Typen - normalize() ergänzt beides, damit dieselbe
        # Zeichen- und Physikroutine sie abspielen kann.
        self._rep_layouts = [gen.normalize(lay)
                             for lay in (rep.get("layouts") or [])]
        self._rep_at = None
        meta = rep.get("meta") or {}
        self.course = meta.get("course", self.course)
        try:
            self.tour = max(1, min(gen.TOUR_COURSES, int(meta.get("tour", 1))))
        except (TypeError, ValueError):
            self.tour = 1
        self.players = max(1, min(2, int(meta.get("players", 1) or 1)))
        self.multiplayer = self.players > 1
        # Bahnen der Runde aus den Szenen ableiten (Reihenfolge = Spielverlauf).
        holes, order = {}, []
        for sc in rep.get("scenes", []):
            h = sc.get("hole", 0)
            if h not in holes:
                idx = sc.get("layout", 0)
                if 0 <= idx < len(self._rep_layouts):
                    holes[h] = self._rep_layouts[idx]
                    order.append(h)
        self.holes = [holes[h] for h in order]
        self._rep_pos = {h: i for i, h in enumerate(order)}
        self.cards = [[0] * len(self.holes) for _ in range(self.players)]
        self.points = [0] * self.players
        self.state = PLAY
        self.phase = "aim"
        self.game_over = False
        self.msg = None
        self.msg_t = 0.0
        self.charging = False
        self.hole_idx = 0
        self.player = 0
        self.strokes = 0
        self.trail = []
        self.replay_seek(0, 0)

    def replay_seek(self, index, frame):
        """Setzt Bahn, Ball und Scorekarte auf Szene 'index', Sample 'frame'."""
        scenes = self._rep.get("scenes", [])
        if not scenes:
            return
        index = max(0, min(len(scenes) - 1, index))
        sc = scenes[index]
        lay = sc.get("layout", 0)
        self.hole = (self._rep_layouts[lay]
                     if 0 <= lay < len(self._rep_layouts) else self.hole)
        if (self.hole["w"], self.hole["h"]) != (self.cw, self.ch):
            self.cw, self.ch = self.hole["w"], self.hole["h"]
            self._layout()
        self.hole_idx = self._rep_pos.get(sc.get("hole", 0), 0)
        self.par = self.hole.get("par", 3)
        self.cup = (float(self.hole["cup"][0]), float(self.hole["cup"][1]))
        self.player = min(self.players - 1, max(0, sc.get("player", 0)))
        self.aim = float(sc.get("aim", 0.0))
        self.power = float(sc.get("power", 0.35))
        self.result_key = sc.get("result")
        self.result_pts = sc.get("pts", 0)

        pts = sc.get("f") or []
        n = max(1, len(pts) // 2)
        frame = max(0, min(n - 1, frame))
        self.bx, self.by = float(pts[2 * frame]), float(pts[2 * frame + 1])
        self.trail = [(float(pts[2 * k]), float(pts[2 * k + 1]))
                      for k in range(max(0, frame - 26), frame)]
        rate = float(self._rep.get("rate") or replay.RATE)
        self.mill_a = float(sc.get("mill", 0.0)) + frame / rate
        self.move_t = float(sc.get("move", 0.0)) + frame / rate
        # Schlagzahl + Scorekarte aus dem bisherigen Verlauf aufbauen.
        last = (frame >= n - 1)
        self.strokes = sc.get("after", sc.get("n", 1)) if last else sc.get("n", 1)
        self.cards = [[0] * len(self.holes) for _ in range(self.players)]
        self.points = [0] * self.players
        for prev in scenes[:index] + ([sc] if last else []):
            fin = prev.get("final")
            p = min(self.players - 1, max(0, prev.get("player", 0)))
            pos = self._rep_pos.get(prev.get("hole", 0))
            if fin and pos is not None:
                self.cards[p][pos] = fin
                self.points[p] += prev.get("pts", 0)
        self.msg = (t("golf.penalty") if (last and sc.get("end") == "water")
                    else None)
        self._rep_at = (index, frame)

    def replay_draw(self, aiming=False, banner=False):
        """Zeichnet den aktuellen Replay-Stand (ohne Menü-Overlay)."""
        s = self.surface
        ui.draw_background(s, self.width, self.height)
        self._draw_course(s)
        self._draw_ball(s)
        if aiming:
            self._draw_aim(s)
        self._draw_hud(s)
        self._draw_card(s)
        if banner and self.result_key:
            self._draw_hole_done(s)

    # ===================================================== Zeichnen
    def draw(self):
        s = self.surface
        # Sicherheitsnetz: In der Pause kommt kein Loslassen der rechten
        # Maustaste mehr an - eine Pause hebt die Stärke-Sperre deshalb selbst
        # auf, statt sie hängen zu lassen.
        if self.power_lock and self.paused:
            self.power_lock = False
        ui.draw_background(s, self.width, self.height)
        if self.state == EDIT and self.editor is not None:
            self.editor.draw(s)
            return
        if self.state == SETUP:
            self._draw_setup(s)
            return
        self._draw_course(s)
        self._draw_ball(s)
        if self.state == PLAY and self.phase == "aim":
            self._draw_aim(s)
        self._draw_hud(s)
        self._draw_card(s)
        if self.state == HOLE_DONE:
            self._draw_hole_done(s)
        elif self.state == OVER:
            self._draw_over(s)

    def _draw_course(self, s):
        """Platz zeichnen - die Arbeit macht minigolf_draw (auch für den Editor)."""
        draw.draw_course(s, self.hole, self._view(), self.mill_a, self.move_t,
                         cup_r=CUP_R)

    def _draw_ball(self, s):
        px, py = self._project(self.bx, self.by)
        r = max(2, int(BR * self.scale))
        for i, (tx, ty) in enumerate(self.trail):
            a = (i + 1) / (len(self.trail) + 1)
            tpx, tpy = self._project(tx, ty)
            pygame.draw.circle(s, ui.mix(COL_GREEN, COL_BALL, a * 0.35),
                               (int(tpx), int(tpy)), max(1, int(r * 0.6)))
        pygame.draw.circle(s, (18, 46, 28), (int(px + 2), int(py + 2)), r)
        pygame.draw.circle(s, COL_BALL, (int(px), int(py)), r)
        pygame.draw.circle(s, (206, 210, 200), (int(px), int(py)), r, 1)
        if r >= 5:
            pygame.draw.circle(s, (255, 255, 255),
                               (int(px - r // 3), int(py - r // 3)),
                               max(1, r // 3))

    def _draw_aim(self, s):
        px, py = self._project(self.bx, self.by)
        ox, oy = math.cos(self.aim), math.sin(self.aim)
        lock = self.power_lock
        col = COL_LOCK if lock else COL_AIM
        if self.guide:
            steps = max(6, int((30 + 60 * self.power) * self.scale / 9))
            for i in range(0, steps, 2):
                pygame.draw.line(s, col,
                                 (px + ox * (i * 9 + 6), py + oy * (i * 9 + 6)),
                                 (px + ox * (i * 9 + 12), py + oy * (i * 9 + 12)),
                                 2)
        # Stärke-Sperre: ruhig pulsender Ring um den Ball - auch ohne Blick
        # aufs HUD ist klar, dass der Schlag geladen wartet.
        if lock:
            puls = 0.5 + 0.5 * math.sin(self.lock_t * 5.0)
            rr = int(max(6.0, BR * self.scale) + 5 + puls * 4)
            pygame.draw.circle(s, COL_LOCK, (int(px), int(py)), rr, 2)
        # Schläger hinter dem Ball
        bx1 = px - ox * (10 + self.power * 26)
        by1 = py - oy * (10 + self.power * 26)
        pygame.draw.line(s, (228, 228, 222), (bx1, by1),
                         (bx1 - ox * 26, by1 - oy * 26), 3)
        pygame.draw.line(s, COL_LOCK if lock else (150, 154, 160), (bx1, by1),
                         (bx1 - oy * 7, by1 + ox * 7), 5)

    def _draw_hud(self, s):
        pygame.draw.rect(s, ui.PANEL, (0, 0, self.width, self.hud_h))
        pygame.draw.line(s, ui.BORDER, (0, self.hud_h), (self.width, self.hud_h))
        cy = self.hud_h // 2
        left = self._tiny.render(
            t("golf.hole", n=self.hole_idx + 1, total=len(self.holes))
            + "  ·  " + t("golf.par", n=self.par), True, ui.TEXT_DIM)
        s.blit(left, left.get_rect(midleft=(12, cy)))
        col = self.accent
        if self.power_lock and self.phase == "aim":
            # Die Sperre ist ein Dauerzustand - sie steht über der Kurzmeldung.
            mid = t("golf.lock", n=self._power_pct())
            col = COL_LOCK
        elif self.msg:
            mid = self.msg
        elif self.multiplayer:
            mid = t("golf.turn",
                    name=t("common.player1" if self.player == 0 else "common.player2"))
        else:
            mid = t("golf.strokes", n=self.strokes)
        img = self._small.render(mid, True, col)
        s.blit(img, img.get_rect(center=(self.width // 2, cy)))
        self._draw_power(s, cy)

    def _power_pct(self):
        """Schlagstärke in ganzen Prozent (Anzeige im HUD)."""
        return int(round(self.power * 100))

    def _draw_power(self, s, cy):
        """Ladebalken rechts im HUD, mit Prozentzahl davor.

        Frei lädt er in der Akzentfarbe. Gesperrt (rechte Maustaste) wird er
        golden, bekommt ein Schloss, eine helle Haltemarke am eingefrorenen
        Wert und einen ruhigen Puls um den Rahmen.
        """
        mw, mh = 84, 10
        mx = self.width - mw - 14
        top = cy - mh // 2
        lock = self.power_lock
        col = COL_LOCK if lock else self.accent
        pygame.draw.rect(s, ui.BTN, (mx, top, mw, mh), border_radius=4)
        fill = int(mw * self.power)
        if fill > 0:
            pygame.draw.rect(s, col, (mx, top, fill, mh), border_radius=4)
        pct = self._tiny.render("%d%%" % self._power_pct(), True,
                                col if lock else ui.TEXT_DIM)
        s.blit(pct, pct.get_rect(midright=(mx - (21 if lock else 6), cy)))
        if not lock:
            return
        self._draw_lock_icon(s, mx - 12, cy, col)
        tick = mx + max(1, min(mw - 1, fill))
        pygame.draw.line(s, (255, 250, 232), (tick, top - 3),
                         (tick, top + mh + 2), 2)
        puls = 0.5 + 0.5 * math.sin(self.lock_t * 5.0)
        pygame.draw.rect(s, ui.mix(col, (255, 255, 255), 0.10 + 0.35 * puls),
                         (mx - 2, top - 2, mw + 4, mh + 4), 1, border_radius=6)

    @staticmethod
    def _draw_lock_icon(s, x, cy, col):
        """Winziges Vorhängeschloss (11x15 px), um (x, cy) zentriert.

        Bügel als Halbkreis mit zwei Beinen, darunter der Körper mit
        Schlüsselloch - so ist das Schloss auch bei 11 px noch als solches
        zu erkennen.
        """
        pygame.draw.arc(s, col, (x - 3, cy - 8, 7, 8), 0.0, math.pi, 2)
        pygame.draw.line(s, col, (x - 3, cy - 5), (x - 3, cy - 1))
        pygame.draw.line(s, col, (x + 3, cy - 5), (x + 3, cy - 1))
        pygame.draw.rect(s, col, (x - 5, cy - 1, 11, 8), border_radius=2)
        pygame.draw.rect(s, ui.PANEL, (x - 1, cy + 2, 2, 3))

    def _draw_card(self, s):
        """Scorekarte rechts: Bahn, Par und Schläge je Spieler."""
        x, w = self.card_x, self.card_w
        y = self.hud_h + 10
        line = self._card.get_height() + 2
        rect = pygame.Rect(x, y, w, (len(self.holes) + 2) * line + 18)
        if rect.bottom > self.height - 4:
            return
        ui.draw_panel(s, rect, radius=8, shadow=False)
        cy = y + 8
        par_x, me_x, p2_x = x + w - 62, x + w - 38, x + w - 16
        s.blit(self._card.render(t("golf.card_hole"), True, ui.TEXT_DIM),
               (x + 8, cy))
        s.blit(self._card.render(t("golf.card_par"), True, ui.TEXT_DIM), (par_x, cy))
        s.blit(self._card.render("1", True, self.accent), (me_x, cy))
        if self.multiplayer:
            s.blit(self._card.render("2", True, ui.GOLD), (p2_x, cy))
        cy += line + 2
        for i, hole in enumerate(self.holes):
            active = (i == self.hole_idx)
            s.blit(self._card.render(str(i + 1), True,
                                     ui.TEXT if active else ui.TEXT_DIM),
                   (x + 8, cy))
            s.blit(self._card.render(str(hole["par"]), True, ui.TEXT_FAINT),
                   (par_x, cy))
            for p, cx in ((0, me_x), (1, p2_x)):
                if p >= self.players:
                    continue
                v = self.cards[p][i]
                if active and p == self.player and self.state == PLAY:
                    v = self.strokes
                if v:
                    col = ui.GREEN if v < hole["par"] else (
                        ui.RED if v > hole["par"] else ui.TEXT)
                    s.blit(self._card.render(str(v), True, col), (cx, cy))
            cy += line
        pygame.draw.line(s, ui.BORDER, (x + 6, cy + 1), (x + w - 6, cy + 1))
        cy += 5
        s.blit(self._card.render(t("golf.card_sum"), True, ui.TEXT_DIM), (x + 8, cy))
        s.blit(self._card.render(str(sum(h["par"] for h in self.holes)), True,
                                 ui.TEXT_FAINT), (par_x, cy))
        for p, cx in ((0, me_x), (1, p2_x)):
            if p < self.players:
                s.blit(self._card.render(str(sum(self.cards[p])), True,
                                         self.accent if p == 0 else ui.GOLD),
                       (cx, cy))

    def _banner(self, s, h=104):
        if self._over_cache is None or self._over_cache.get_width() != self.width \
                or self._over_cache.get_height() != h:
            ov = pygame.Surface((self.width, h), pygame.SRCALPHA)
            ov.fill((8, 14, 10, 214))
            self._over_cache = ov
        y = self.height // 2 - h // 2
        s.blit(self._over_cache, (0, y))
        pygame.draw.line(s, self.accent, (0, y), (self.width, y))
        pygame.draw.line(s, self.accent, (0, y + h - 1), (self.width, y + h - 1))
        return y

    def _draw_hole_done(self, s):
        y = self._banner(s)
        cx = self.width // 2
        head = self._huge.render(t(self.result_key), True, self.accent)
        s.blit(head, head.get_rect(center=(cx, y + 34)))
        sub = self._small.render(
            t("golf.hole_result", strokes=self.strokes, par=self.par,
              pts=self.result_pts), True, ui.TEXT)
        s.blit(sub, sub.get_rect(center=(cx, y + 66)))
        hint = self._tiny.render(t("golf.next"), True, ui.TEXT_DIM)
        s.blit(hint, hint.get_rect(center=(cx, y + 90)))

    def _draw_over(self, s):
        y = self._banner(s, self.over_h)
        cx = self.width // 2
        total = sum(self.cards[0])
        par_total = sum(h["par"] for h in self.holes)
        if self.multiplayer:
            head = self._huge.render(
                t("common.draw") if self.winner is None
                else t("common.player_wins", n=self.winner + 1), True, self.accent)
        else:
            head = self._huge.render(t("golf.round_done"), True, self.accent)
        s.blit(head, head.get_rect(center=(cx, y + self.over_y["head"])))
        d = total - par_total
        sub = self._small.render(
            t("golf.final", strokes=total,
              diff=("%+d" % d) if d else t("golf.even"), pts=self.points[0]),
            True, ui.TEXT)
        s.blit(sub, sub.get_rect(center=(cx, y + self.over_y["sub"])))
        best = self.best.get(self._best_key())
        if best:
            b = self._tiny.render(t("golf.best", n=best), True, ui.GOLD)
            s.blit(b, b.get_rect(center=(cx, y + self.over_y["best"])))
        # Knopfreihe: Weiter ist der hervorgehobene Standardweg.
        labels = {"next": t("golf.btn_next", course=self._next_course_label()),
                  "again": t("golf.btn_again"), "setup": t("golf.btn_setup"),
                  "replay": t("golf.btn_replay")}
        for key, rc in self.over_rects:
            self._btn(s, rc, labels[key], key == self.over_rects[0][0])
        # Tastenzeile passend zu den vorhandenen Knöpfen.
        hint_key = ("golf.continue_hint" if self._next_course()
                    else "golf.new_round")
        hint_txt = t(hint_key)
        if self.replay is not None:
            hint_txt += "  ·  " + t("golf.replay_hint")
        hint = self._tiny.render(hint_txt, True, ui.TEXT_DIM)
        s.blit(hint, hint.get_rect(center=(cx, y + self.over_y["hint"])))

    def _draw_setup(self, s):
        cx = self.width // 2
        title = self._huge.render(t("golf.title"), True, self.accent)
        s.blit(title, title.get_rect(center=(cx, int(self.height * 0.115))))
        sub_key = ("golf.subtitle" if self.setup_tab == "play"
                   else "golf.ugc.subtitle")
        sub = self._small.render(t(sub_key), True, ui.TEXT_DIM)
        s.blit(sub, sub.get_rect(center=(cx, int(self.height * 0.18))))
        for i, rc in enumerate(self.tab_rects):
            self._btn(s, rc, t("golf.tab_" + TABS[i]),
                      self.setup_tab == TABS[i])
        if self.setup_tab == "maps":
            if self.maps is None:
                from . import minigolf_edit as edit
                self.maps = edit.MapList(self)
            self.maps.draw(s)
            return

        def label(rects, txt, w=None):
            """Beschriftung mittig über die Gruppe (zu lange wird gekürzt)."""
            im = self._tiny.render(txt, True, ui.TEXT_DIM)
            if w and im.get_width() > w:
                kurz = txt
                while len(kurz) > 2 and self._tiny.size(kurz + "...")[0] > w:
                    kurz = kurz[:-1]
                im = self._tiny.render(kurz + "...", True, ui.TEXT_DIM)
            mid = (rects[0].left + rects[-1].right) // 2
            s.blit(im, im.get_rect(midbottom=(mid, rects[0].top - 4)))

        label(self.course_rects, t("golf.lbl_course"))
        for i, rc in enumerate(self.course_rects):
            self._btn(s, rc, t("golf.course." + COURSES[i]),
                      self.course == COURSES[i])
        self._draw_tour_row(s)
        # Schalterzeile: Ziellinie · Autoziel · Aufnehmen
        for rects, key, on in (
                (self.guide_rects, "golf.lbl_guide", self.guide),
                (self.autoaim_rects, "golf.lbl_autoaim", self.autoaim),
                (self.pickup_rects, "golf.lbl_pickup", self.pickup)):
            label(rects, t(key), self.opt_w)
            for i, rc in enumerate(rects):
                self._btn(s, rc, t("common.on") if i == 0 else t("common.off"),
                          on == (i == 0))
        pygame.draw.rect(s, ui.BTN_SEL, self.start_rect, border_radius=9)
        pygame.draw.rect(s, self.accent, self.start_rect, 2, border_radius=9)
        st = self.font.render(t("common.start"), True, ui.TEXT)
        s.blit(st, st.get_rect(center=self.start_rect.center))
        best = self.best.get(self._best_key())
        if best:
            b = self._tiny.render(t("golf.best", n=best), True, ui.GOLD)
            s.blit(b, b.get_rect(center=(cx, self.start_rect.bottom + 14)))
        hint = self._tiny.render(t("golf.setup_hint"), True, ui.TEXT_DIM)
        s.blit(hint, hint.get_rect(center=(cx, self.height - 12)))

    def _draw_tour_row(self, s):
        """Kurswahl der Tour: Pfeile, Kursnummer und Gesamt-Par."""
        left, mid, right = self.tour_rects
        on = (self.course == "tour")
        im = self._tiny.render(t("golf.lbl_tour"), True,
                               ui.TEXT_DIM if on else ui.TEXT_FAINT)
        s.blit(im, im.get_rect(midbottom=(self.width // 2, left.top - 3)))
        col = self.accent if on else ui.BORDER
        for rc, arrow in ((left, "<"), (right, ">")):
            pygame.draw.rect(s, ui.BTN, rc, border_radius=7)
            pygame.draw.rect(s, col, rc, 1, border_radius=7)
            a = self._small.render(arrow, True, ui.TEXT if on else ui.TEXT_FAINT)
            s.blit(a, a.get_rect(center=rc.center))
        pygame.draw.rect(s, ui.BTN_SEL if on else ui.BTN, mid, border_radius=7)
        pygame.draw.rect(s, col, mid, 2 if on else 1, border_radius=7)
        txt = t("golf.tour", n=self.tour, total=gen.TOUR_COURSES)
        if on:
            txt += "  ·  " + t("golf.par", n=self._tour_par)
        im = self._small.render(txt, True, ui.TEXT if on else ui.TEXT_FAINT)
        s.blit(im, im.get_rect(center=mid.center))

    def _btn(self, s, rc, text, on):
        pygame.draw.rect(s, ui.BTN_SEL if on else ui.BTN, rc, border_radius=8)
        pygame.draw.rect(s, self.accent if on else ui.BORDER, rc,
                         2 if on else 1, border_radius=8)
        col = ui.TEXT if on else ui.TEXT_DIM
        im = self._small.render(text, True, col)
        if im.get_width() > rc.w - 12:     # z.B. "Weiter: Tour 12"
            im = self._tiny.render(text, True, col)
        if im.get_width() > rc.w - 8:      # vier Knöpfe auf 480 px: kürzen
            kurz = text
            while len(kurz) > 2 and self._tiny.size(kurz + "...")[0] > rc.w - 8:
                kurz = kurz[:-1]
            im = self._tiny.render(kurz + "...", True, col)
        s.blit(im, im.get_rect(center=rc.center))
