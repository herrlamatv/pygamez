# -*- coding: utf-8 -*-
"""
regression.py
=============
Alle portierten Spiele erneut gegen Python pruefen (nach Aenderungen an
py.rs / ui.rs / pg.rs, die viele Spiele treffen).

    .venv/Scripts/python.exe rust/tools/regression.py [--aufloesung 1280x960] [--sprache en] [spiel ...|screens]

Je Spiel laufen ein bis drei feste Ereignisfolgen durch ``vergleich.py``
(gleicher Startwert, gleiche Uhr). Die Folgen sind bewusst einfach: Start,
ein paar Spielzuege, Mausklicks - sie decken Zeichnen, Zufall und Logik ab,
aber nicht jede Spielsituation (dafuer gibt es die Bots in ``bots/``).

Ausgabe: eine Zeile je Lauf, am Ende die Zahl der Abweichungen.
"""

import json
import os
import shutil
import subprocess
import sys
import tempfile

HIER = os.path.dirname(os.path.abspath(__file__))
WURZEL = os.path.dirname(os.path.dirname(HIER))


def folge(tokens, abstand):
    """'Return _ Up' -> Liste mit ``abstand`` leeren Bildern nach jedem Eintrag."""
    out = []
    for k in tokens.split():
        out.append("" if k == "_" else k)
        out.extend([""] * abstand)
    return out


def wdh(muster, n):
    return " ".join([muster] * n)


def spur(tokens):
    """'_*20 space _*3 Return' -> '_ _ ... space _ _ _ Return' (fuer Abstand 0,
    z.B. Bot-Spuren mit genauen Bildnummern)."""
    out = []
    for k in tokens.split():
        out.append(wdh("_", int(k[2:])) if k.startswith("_*") else k)
    return " ".join(out)


# spiel: [(modus, bilder, tokens, abstand), ...]
LAEUFE = {
    "pong": [("single", 600, "Return " + wdh("+Up _ -Up +Down _ -Down", 8), 12),
             # Pause-Overlay (main._draw_pause_overlay) ueber dem Spielbild
             ("single", 120, "Return " + wdh("_", 30) + " Escape", 1)],
    "slide": [("single", 300, "Return " + wdh("Up Left Down Right", 10), 4)],
    "hangman": [("single", 300, "Return e n i s r a t d h u l c g m", 8)],
    "mastermind": [("single", 300, "Return 1 2 3 4 Return 2 3 4 5 Return 3 4 5 6 Return", 6)],
    "bubble": [("single", 500, "Return " + wdh("Left Left space Right Right Right space", 6), 10)],
    "kniffel": [("single", 300, "Return space 1 3 space space @320:300 space", 10)],
    "memory": [("single", 300, "Return @100:150 @200:150 @300:150 @400:150 @100:250 @200:250", 18)],
    "simon": [("single", 400, "Return", 4)],
    "connect4": [("single", 600, "Return 4 3 4 5 2 6 1 7 4 4", 25)],
    "trex": [("single", 700, "Return " + wdh("space _ _ Down", 10), 14)],
    "tictactoe": [("single", 400, "Return 5 1 9 3 7 2 4 6 8", 20)],
    "minesweeper": [("single", 300, "Return @320:240 @200:200 @400:300 @250:350:3 @100:100", 12)],
    "maze": [("single", 500, "Return " + wdh("+Up _ -Up +Right _ -Right +Down _ -Down +Left _ -Left", 4), 10)],
    "reversi": [("single", 600, "Return @300:200 @340:240 @260:280 @380:200 @220:240", 40)],
    "flappy": [("single", 500, "Return " + wdh("space _ _ _", 20), 5)],
    "blackjack": [("single", 600, "Return Return h s Return Return d Return Return s Return", 30)],
    "frogger": [("single", 500, "Return " + wdh("Up _ Left Up Right", 12), 6)],
    "doodle": [("single", 600, "Return " + wdh("+Left _ _ -Left +Right _ _ -Right", 8), 10)],
    "muehle": [("single", 600, "Return @320:240 @200:200 @440:280 @260:120 @380:360", 60)],
    "tanks": [("single", 600, "Return " + wdh("+Up _ -Up +Left _ -Left space", 8), 10)],
    "dame": [("single", 500, "Return @100:300 @150:250 @200:300 @250:250", 50)],
    "tunnel": [("endless", 600, "Return " + wdh("+Left _ -Left +Right _ -Right", 10), 8),
               ("levels", 400, "Return Return " + wdh("+Up _ -Up", 6), 10)],
    "airhockey": [("single", 600, "Return " + wdh("~200:400 ~320:380 ~440:420", 20), 8)],
    "asteroids": [("single", 600, "Return " + wdh("+Up _ -Up +Left _ -Left space space", 10), 6)],
    "invaders": [("classic", 600, "Return " + wdh("+Left _ -Left space +Right _ -Right space", 10), 6),
                 ("free", 400, "Return " + wdh("+Right _ -Right space", 10), 6)],
    "pacman": [("single", 600, "Return " + wdh("Left _ Up _ Right _ Down", 10), 8)],
    "bowling": [("single", 700, "Return space _ space _ _ _ space _ space", 40)],
    "poker": [("holdem", 700, "Return c c r c c c Return Return c f Return Return a c", 45),
              ("draw", 500, "Return c c c 1 3 Return c c c Return Return", 30),
              ("video", 300, "3 Return 1 2 Return Return 2 Return 3 4 5 Return Return", 12)],
    "breakout": [("single", 900, "Return _ _ _ _ space " + wdh("~250:444 ~380:444", 30), 12)],
    "aim": [("precision", 900, "Return " + wdh("^30:-10 @320:240 ^-50:20 @320:240", 12), 15),
            ("chill", 600, "3 b Return " + wdh("^-40:5 @320:240", 12), 20)],
    "pinball": [("single", 900, "Return +space _ _ _ -space " + wdh("+Left _ -Left +Right _ -Right", 12), 10),
                ("multi", 600, "2 Return +space _ _ -space " + wdh("+Right _ -Right Up", 8), 12)],
    "solitaire": [("klondike", 500, "Return space space @100:300 @200:300 u <300:250 ~320:270 ~350:300 >350:300 @300:250:3", 20),
                  ("spider", 400, "Return space @150:300 @250:300 u", 25),
                  ("freecell", 400, "Return @120:300 @60:80 <200:320 ~150:200 >60:80", 20),
                  ("pyramid", 400, "Return space @320:90 @300:140 space", 25),
                  ("tripeaks", 400, "Return space @150:250 space @320:250", 25)],
    "sudoku": [("comfort", 600, "Return _ _ @100:100 5 @150:150 3 u y n @200:200 4 n h c", 15),
               ("classic", 400, "v v Return _ _ @120:120 7 @160:160 2 @200:200:3", 15),
               ("assist", 400, "m t _ _ @100:150 9 h h m", 15)],
    "billiard": [("single", 900, "Return ~400:200 <400:200 _ _ _ _ _ >400:200 " + wdh("~300:250 <300:250 _ _ >300:250", 6), 25),
                 ("multi", 600, "2 v Return ~420:220 <420:220 _ _ >420:220 v", 30)],
    "blockjump": [("normal", 900, "Return +w " + wdh("space ^5:-2 _ ^-3:1", 20) + " v t b -w", 8),
                  ("hard", 500, "Return +d " + wdh("space ^-4:0", 15) + " -d", 10)],
    "2048": [("single", 700, "Return " + wdh("Left Up Right Down u", 25), 3),
             ("single", 500, "3 m Return " + wdh("Down Left <100:200 _ >200:200", 15), 4)],
    "lamatowerdef": [("classic", 1500, "Return f 1 @152:245 2 @287:245 space _ _ @152:245 u _ g", 30),
                     ("maximal", 1200, "Return 1 @152:245 1 @395:272 space q ~200:191 @200:191 w e", 40),
                     ("compact", 600, "Right Return Left Return 4 @152:245 x @152:245 @152:245:3", 20)],
    "wordle": [("endless", 700, "Return s o n n e Return _ _ _ _ a b Return BackSpace BackSpace h a u s e Return _ _ _ _ "
                                "@621:17 Down space", 8),
               ("daily", 500, "Right Return t a g e n Return _ _ _ _ _ x x x x x Return", 8),
               ("dordle", 500, "f h Return b l u m e Return _ _ _ _ _ s t e r n Return", 8),
               ("quordle", 500, "4 Return h a u s Return _ _ _ _ _ @621:17", 8)],
    "battleship": [("single", 900, "Return x Return @49:87 _ _ _ _ _ _ @85:123 _ _ _ _ _ _ @193:231 Up space", 20),
                   ("multi", 600, "Return Return x Return Return Return x Return Return @121:159", 25)],
    "crossy": [("endless", 900, "Return _ _ Up _ _ Up _ _ _ Up _ Left _ Up _ Right Up Up", 12),
               ("daily", 500, "Tab Right Down Return Escape h n Return Up _ Up", 15)],
    "snake": [("single", 700, "Return _ _ _ s _ _ d _ _ +space _ _ _ -space w _ a", 15),
              # Rundenende -> P (Replay-Player) -> e (Export nach "Downloads")
              ("single", 1050, "Return _ _ _ s _ _ d _ _ +space _ _ _ -space w _ a " + wdh("_", 20)
               + " P _ _ e " + wdh("_", 18), 15),
              ("single", 600, "v Return _ a _ _ _ d _ _ +space _ _ -space", 20),
              ("multi", 500, "Return _ _ s Down _ _ d Right", 15),
              ("single", 400, "6 h Return _ s _ d", 20),
              ("single", 300, "c Tab 7 Escape v k Right Up Escape 3 Return", 10)],
    "tetris": [("solo", 900, "Return _ _ _ _ _ _ " + wdh("Left Up space Right Right z space c Down space", 6), 8),
               ("solo", 600, "2 Down Down Left Return _ _ _ _ _ _ " + wdh("x Left Left space", 10), 6),
               ("versus_ai", 900, "3 Return _ _ _ _ _ _ " + wdh("space Up Right space", 15), 10),
               ("multi", 700, "Return _ _ _ _ _ _ " + wdh("a space Left Return w e q Shift_R", 8), 8)],
    "casino": [("roulette", 900, "Return 5 @300:400 ~200:300 @200:300 3 @250:350 @250:350:3 d space _ _ _ _ "
                                 "space _ _ _ _ _ _ _ _ _ _ _ r space", 20),
               ("slots", 900, "t space _ _ _ _ _ i _ i Up Up a _ _ _ _ _ _ _ _ _ _ a ~50:110 Down space", 15)],
    "chess": [("single", 700, "Down Down Down Right Return _ _ _ Return Up Up Return _ _ _ _ h _ _ _ _ f", 10),
              # ... Remis angeboten/abgelehnt, x = Aufgeben, p = PGN-Export
              ("multi", 600, "Down Down Right Return _ _ Return Up Up Return _ _ _ Return Down Down Return _ o n "
                             "x Return _ _ p _ _", 12),
              ("puzzles", 600, "2 Right Return _ _ _ _ _ h _ _ _ _ _ _ _ _ n", 15)],
    "geodash": [("normal", 900, "Return " + wdh("+space _ -space _ _ _ _", 25), 3),
                ("practice", 800, "3 Return _ _ z _ +Up _ _ -Up z x _ p _ p _ r " + wdh("<320:240 _ >320:240 _ _", 10), 6),
                ("normal", 700, "Tab _ c c Down Down f a BackSpace Return Down Return _ _ " + wdh("+space _ -space _ _", 10), 8),
                # Editor -> Bibliothek -> Teilen (Downloads) -> Import (ohne Dialog: Hinweis)
                ("normal", 820, "ISO_Left_Tab _ n _ Right Right Return _ _ 3 @150:250 @190:250 r @230:300 u y g 5 @300:250 "
                                "@330:250:3 s Return _ _ _ _ Escape Escape @320:450 _ _ Return _ _ _ _ @527:450 _ _ _ _", 12)],
    # Lama Crossing 2.0 (Spuren aus bots/bot_lamacrossing.py im Python-Pruefstand,
    # 640x480, Startwert 7): Strasse/Mittel - Raser (Zufalls-Fehltritt) auf Spur 1, dann
    # 5 Spuren durch den Verkehr + Auszahlen (Baken, haltende Autos); Raser
    # genau bei der Landung; Schwer - ins Auto gelaufen (platt + Reifenspuren);
    # Bruecke bis die Planke bricht; Fluss mit 50er-Einsatz - vom Treibholz
    # erwischt; Eis per Maus (Welt, Stufe, Einsatz +, LOS, naechste Spur,
    # AUSZAHLEN); Eis bis zum Einbrechen (Risse); Pleite mit 1000er-Einsatz auf
    # Extrem (Raser) + Kredit; Steinschlag auf der Bruecke, Eisblock auf dem See
    "lamacrossing": [("single", 560, spur("_*20 space _*289 space _*21 space _*21 space _*47 space _*21 space "
                                          "_*31 Return"), 0),
                     ("single", 46, spur("_*20 space"), 0),
                     ("single", 140, spur("_*10 3 _*50 space"), 0),
                     ("single", 290, spur("_*10 w _*10 " + wdh("space _*21", 9) + " space"), 0),
                     ("single", 200, spur("_*10 w w Up Up _*10 space _*21 space _*21 space _*56 space"), 0),
                     ("single", 200, spur("_*6 ~119:56 _*4 @119:56 _*6 @383:406 _*6 @111:454 _*6 ~587:454 _*4 "
                                          "@587:454 _*21 @301:242 _*21 @308:242 _*47 @299:242 _*27 ~457:454 "
                                          "_*5 @457:454"), 0),
                     ("single", 305, spur("_*10 w w w _*27 space _*21 space _*31 space _*30 space _*21 space "
                                          "_*39 space _*21 space _*21 space _*37 space"), 0),
                     ("single", 400, spur("_*10 Up Up Up Up Up Up 4 _*10 space _*21 space _*319 Return"), 0),
                     ("single", 110, spur("_*10 w _*66 space"), 0),
                     ("single", 160, spur("_*10 w w w Up _*26 space _*89 space"), 0)],
    # PLATZHALTER (RUST-INFRA): Lama Mines / Lama Plinko zeichnen nur ihren Namen.
    # RUST-MINES bzw. RUST-PLINKO ersetzen je ihre Zeile durch echte Laeufe.
    # Lama Mines (Spuren aus bots/bot_lamamines.py, 640x480, Startwert 7): Runde per
    # Tastatur (Cursor, R = Zufallsfeld, Enter, Leertaste = naechste Runde); Mine per
    # Maus - Explosion, BUMM-Tafel mit Rest-Aufdeckung, danach Ruhe; Einstellungen per
    # Tasten/Maus/Mausrad + Zufallsfeld + AUSZAHLEN + Tooltip; 4x4/8 Minen abgeraeumt;
    # Hoechstgewinn x100.000 (5x5/16 Minen); Auto-Fenster (Markierungen, Optionen,
    # Turbo) und Auto-Lauf; Statistik; Fairness (Nachrechnen, Liste, Mausrad, neuer
    # Client-Seed); Pleite mit 1000er-Einsatz + Kredit
    "lamamines": [("single", 300, spur("_*5 space _*19 Left Left Left Up Up space _*14 Right space _*14 Right space r "
                                       "_*14 Down Down Right Right Up Up Up space Return _*3 space _*16 space"), 0),
                  ("single", 90, spur("_*5 ~300:300 _*3 @587:454 _*19 @53:122 _*14 @116:122 _*14 @242:122"), 0),
                  ("single", 160, spur("_*5 ~300:300 _*3 @587:454 _*19 @53:122 _*14 @116:122 _*14 @242:122 "
                                       "_*30 space"), 0),
                  ("single", 300, spur("_*5 ~300:300 _*3 @587:454 _*19 @53:122 _*14 @116:122 _*14 @242:122 "
                                       "_*30 space"), 0),
                  ("single", 400, spur("_*4 3 _*4 Right Right _*3 Up Up _*3 %300:300:1 _*3 %300:300:-1 _*3 @111:454 "
                                       "_*3 @24:454 _*3 1 _*3 4 _*3 @462:192 _*4 @611:221 _*3 @549:221 _*3 @559:246 "
                                       "_*3 ~473:246 _*3 @457:454 _*5 @200:200 _*5 ~300:200 _*3 @587:454 _*19 @252:454 "
                                       "_*14 @305:122 ~457:454 _*3 @457:454 _*20 ~60:55"), 0),
                  ("single", 200, spur("_*4 1 _*3 @559:246 _*3 space _*18 @219:130 _*14 @61:209 _*14 @140:209 "
                                       "_*14 @219:209 _*14 @140:288 _*14 @298:288 _*14 @219:367 _*14 @298:367"), 0),
                  ("single", 200, spur("_*4 @559:246 _*2 Right _*3 space _*19 @305:122 _*14 @116:185 _*14 @116:248 "
                                       "_*14 @116:311 _*14 @179:311 _*14 @242:311 _*14 @305:311 _*14 @53:374"), 0),
                  ("single", 60, spur("_*4 a _*4 @53:122 _*2 @116:185 _*2 ~179:248 _*3 @179:248 _*3 @179:248 _*3 "
                                      "Right space Right space _*3 @392:133 _*2 @495:172 _*2 @421:211 _*3 t"), 0),
                  ("single", 300, spur("_*4 a _*4 @53:122 _*2 @116:185 _*2 ~179:248 _*3 @179:248 _*3 @179:248 _*3 "
                                       "Right space Right space _*3 @392:133 _*2 @495:172 _*2 @421:211 _*3 t _*3 "
                                       "@531:399"), 0),
                  ("single", 420, spur("_*4 a _*4 @53:122 _*2 @116:185 _*2 @179:248 _*2 @242:311 _*3 @392:133 _*2 t "
                                       "_*3 Return _*367 s _*10 ~300:350"), 0),
                  ("single", 800, spur("_*4 a _*4 @53:122 _*2 @116:185 _*2 @179:248 _*2 @242:311 _*3 @392:133 _*2 t "
                                       "_*3 Return _*367 s _*10 ~300:350 _*10 ~120:120 _*5 Escape _*5 @110:55 _*5 "
                                       "@192:55 _*10 @319:245 _*10 @152:191 _*5 @319:245 _*5 @152:172 _*3 @319:245 "
                                       "_*5 %200:300:-1 _*5 %200:300:1 _*5 @536:124 _*10 @10:10 _*5 f _*5 ~152:153"), 0),
                  ("single", 140, spur("_*4 Up Up Up Up Up Up _*3 space _*19 @53:122 _*14 Left Right Right Right "
                                       "space _*75 Return"), 0),
                  ("single", 400, spur("_*4 Up Up Up Up Up Up _*3 space _*19 @53:122 _*14 Left Right Right Right "
                                       "space _*75 Return _*30 @300:300 _*20 space"), 0)],
    # Lama Plinko (Spuren aus bots/bot_lamaplinko.py, 640x480, Startwert 7): zwei Kugeln,
    # Landung mit Popup, Tooltips (Fach, Knopf); Risiko/Reihen/Einsatz per Tasten, Sperre
    # im Flug; Dauerfeuer 16 Reihen + Turbo (Wollknaeuel-Regen); alle Knoepfe per Maus,
    # Fenster auf/zu, Mausrad; Auto per Tastatur (Turbo, Gewinnziel) und Maus (10 Kugeln,
    # Verlustgrenze, Esc-/Leertaste-Stopp); Statistik (leer, Kacheln, Kurve, Verteilung,
    # Balken-Tooltip, Reihen +/-); Fairness (leer, gesperrter Client-Seed, Nachrechnen,
    # Liste, Mausrad, neuer Client-Seed); x32 per Dauerfeuer (Muenzregen); 8 Reihen/Hoch
    # mit 1000er-Einsatz bis pleite + Kredit. Jackpot x1000, Pleite beim Start, kaputter
    # Spielstand, Fortsetzen: per vergleich.py --daten (siehe rust/README.md).
    "lamaplinko": [("single", 200, spur("_*20 space _*30 space _*128 ~320:381 _*10 ~308:454"), 0),
                   ("single", 200, spur("_*10 3 Left Left Left Left Up Up space _*5 Right 1 _*10 space minus Down "
                                        "_*95 Right 2 plus equal KP_Add KP_Subtract KP_3 _*8 space"), 0),
                   ("single", 170, spur("_*10 t Right Right Right Right +space _*20 " + wdh("*space _*1", 60)
                                        + " -space"), 0),
                   ("single", 170, spur("_*6 @69:131 _*4 @28:176 _*4 @28:176 _*4 @111:454 _*4 @24:454 _*4 @24:454 "
                                        "_*4 @579:454 _*20 @320:200 _*6 @110:176 _*6 @380:454 _*6 @272:454 _*10 "
                                        "@566:118 _*6 @308:454 _*10 @5:5 _*6 @344:454 _*10 @566:64 _*6 %320:240:1 "
                                        "_*4 %320:240:1 _*4 %320:240:-1 ~579:454 _*6 ~28:176 _*4 ~272:454"), 0),
                   ("single", 520, spur("_*10 a _*6 Up Right Down Down Right Right Down Left Return _*274 @272:454 "
                                        "_*4 @99:165 @172:259 @172:259 @156:292 _*4 @319:351 _*40 Escape _*122 a "
                                        "_*4 Return _*20 space"), 0),
                   ("single", 150, spur("_*6 s _*10 s t " + wdh("space _*8", 5) + " space _*67 s _*10 ~315:327"), 0),
                   ("single", 185, spur("_*6 s _*10 s t " + wdh("space _*8", 5) + " space _*67 s _*10 ~315:327 _*6 "
                                        "@542:222 _*4 Left _*4 Right Right Right ~259:327 _*8 @565:222 _*6 Left"), 0),
                   ("single", 310, spur("_*6 f _*10 f _*4 space _*10 space _*10 space _*10 f _*10 c _*100 Escape "
                                        "_*10 f _*6 Return _*10 Down Return _*10 %320:240:-1 _*6 @151:202 _*6 "
                                        "@316:399 _*10 %320:240:1 _*4 c _*20 @509:122 _*20 Escape _*10 f _*6 Up "
                                        "Right"), 0),
                   ("single", 210, spur("_*10 3 Left Left Left Left t +space " + wdh("*space", 144) + " -space"), 0),
                   ("single", 1270, spur("_*10 Up Up Up Up Up Up 3 Left Left Left Left t _*5 " + wdh("space _*48", 23)
                                         + " space _*78 Return"), 0)],
    "minigolf": [("single", 900, "Return _ _ _ Up Up Up space _ _ _ _ _ _ _ _ _ _ _ _ g z Down space", 20),
                 ("single", 700, "3 Right Right Return _ _ <300:200 _ _ _ >300:200 _ _ _ _ _ _ f", 15),
                 # Editor -> Speichern -> Bibliothek -> Teilen (Downloads) -> Import
                 ("multi", 1000, "Tab n _ @540:160 <200:200 ~260:240 >260:240 @560:430 Up @190:19 B a h n @575:19 _ _ _ "
                                 "@46:19 _ _ _ @320:444 _ _ Return _ _ _ _ @527:444 " + wdh("_", 15), 12)],
}


# Bildschirme (keine Spiele): (schluessel, bilder, tokens, abstand, testdaten?)
BILDSCHIRME = [
    ("screen:profiles", 160, "n L a m a _ Right Return _ _ n B o b Left Return _ Up Up F2 _ BackSpace x "
                             "@130:230 @200:290 Return _ Down Delete _ ~520:270 _ n _ Escape", 1, False),
    ("screen:levelintro", 30, "", 1, True),
    ("screen:pick:pong", 40, "Down Up ~320:200", 4, True),
    ("screen:progress", 60, "~100:300 Down Down ~300:200 Next ~150:400 @80:143 _ ~600:143 Down", 1, True),
    ("screen:progress:stats", 60, "Next ~300:200 ~200:300 ~400:260 Next ~300:250", 2, True),
    ("screen:progress:calendar", 60, "Next ~300:200 ~520:330 ~320:240 Next ~320:300", 2, True),
    ("screen:progress:timeline", 60, "~100:143 @180:143 _ ~300:200 Next", 2, True),
    ("screen:progress:profiles", 60, "~200:300 Down Next Tab Right Left", 2, True),
    ("screen:wiki", 80, "s n a _ k e _ BackSpace ~100:150 @100:150 Next _ Escape _ t e t r i s Down Next", 1, False),
    ("screen:wiki:snake", 40, "Next Next Down ~100:200 Up Prior", 2, False),
    ("screen:wiki:lamacrossing", 60, "Down Down Next Down", 3, False),
    ("screen:wiki:lamamines", 60, "Next Next Down Up Next", 3, False),
    ("screen:wiki:lamaplinko", 60, "Down Down Next Down", 3, False),
    ("screen:replays", 40, "Left Left Tab Right", 3, False),
    ("screen:menu", 90, "~320:330 _ _ ~150:360 _ _ ~480:300", 4, True),
    ("screen:menu", 60, "~320:330 _ ~150:360", 4, "classic"),
]


def main():
    nur = sys.argv[1:]
    aufloesung = None
    sprache = None
    while len(nur) >= 2 and nur[0] in ("--aufloesung", "--sprache"):
        if nur[0] == "--aufloesung":
            aw, ah = nur[1].lower().split("x")
            aufloesung = (int(aw), int(ah))
        else:
            sprache = nur[1]
        nur = nur[2:]
    exe = os.path.join(WURZEL, ".venv", "Scripts", "python.exe")
    if not os.path.exists(exe):
        exe = sys.executable
    ordner = tempfile.mkdtemp(prefix="pgz_regression_")
    schlecht = 0

    def mit_aufloesung(quelle, name, design=None):
        """Datenordner: Dateien aus 'quelle' (oder leer) + settings.json."""
        ziel = os.path.join(ordner, name)
        os.makedirs(ziel, exist_ok=True)
        if quelle:
            for datei in os.listdir(quelle):
                shutil.copyfile(os.path.join(quelle, datei), os.path.join(ziel, datei))
        st = {}
        if aufloesung:
            st["resolution"] = list(aufloesung)
        if design:
            st["theme"] = design
        with open(os.path.join(ziel, "settings.json"), "w", encoding="utf-8") as f:
            json.dump(st, f)
        if sprache:
            mem_pfad = os.path.join(ziel, "mem.json")
            mem = {"mem": {"lang": "de"}}
            if os.path.exists(mem_pfad):
                with open(mem_pfad, encoding="utf-8") as f:
                    mem = json.load(f)
            mem.setdefault("mem", {})["lang"] = sprache
            with open(mem_pfad, "w", encoding="utf-8") as f:
                json.dump(mem, f)
        return ziel
    eigen = aufloesung or sprache
    spiel_daten = ["--daten", mit_aufloesung(None, "aufloesung")] if eigen else []
    for spiel, laeufe in LAEUFE.items():
        if nur and spiel not in nur:
            continue
        # Jeder Lauf zusaetzlich nach 8 Bildern: deckt den Bildschirm-Uebergang ab.
        laeufe = laeufe + [(laeufe[0][0], 8, laeufe[0][2], laeufe[0][3])]
        for i, (modus, bilder, tokens, abstand) in enumerate(laeufe):
            liste = os.path.join(ordner, "%s_%d.txt" % (spiel, i))
            with open(liste, "w", encoding="utf-8") as f:
                f.write(",".join(folge(tokens, abstand)))
            r = subprocess.run([exe, os.path.join(HIER, "vergleich.py"), spiel,
                                "--mode", modus, "--frames", str(bilder),
                                "--keys-file", liste,
                                "--out", os.path.join(ordner, "%s_%d" % (spiel, i))] + spiel_daten,
                               cwd=WURZEL, capture_output=True, text=True)
            zeile = (r.stdout.strip().splitlines() or [r.stderr.strip()[-200:]])[0]
            ok = r.returncode == 0 and "gleich" in zeile
            schlecht += not ok
            print("%-6s %-12s %-8s %s" % ("ok" if ok else "FEHLER", spiel, modus,
                                          zeile if not ok else ""), flush=True)
    daten = None
    for nr, (key, bilder, tokens, abstand, testdaten) in enumerate(BILDSCHIRME):
        if nur and key not in nur and "screens" not in nur:
            continue
        extra = []
        if testdaten:
            if daten is None:
                daten = os.path.join(ordner, "testdaten")
                subprocess.run([exe, os.path.join(HIER, "testdaten.py"), daten], cwd=WURZEL,
                               capture_output=True, text=True)
            extra = ["--daten", daten]
            if isinstance(testdaten, str) or eigen:
                # Eigenes Design bzw. eigene Aufloesung: Testdaten + settings.json.
                design = testdaten if isinstance(testdaten, str) else None
                extra = ["--daten", mit_aufloesung(daten, "testdaten_%d" % nr, design)]
        elif eigen:
            extra = spiel_daten
        datei = "%s_%d" % (key.replace(":", "_"), nr)
        liste = os.path.join(ordner, datei + ".txt")
        with open(liste, "w", encoding="utf-8") as f:
            f.write(",".join(folge(tokens, abstand)))
        r = subprocess.run([exe, os.path.join(HIER, "vergleich.py"), key, "--frames", str(bilder),
                            "--keys-file", liste, "--out", os.path.join(ordner, datei)] + extra,
                           cwd=WURZEL, capture_output=True, text=True)
        zeile = (r.stdout.strip().splitlines() or [r.stderr.strip()[-200:]])[0]
        ok = r.returncode == 0 and "gleich" in zeile
        schlecht += not ok
        print("%-6s %-26s %s" % ("ok" if ok else "FEHLER", key, zeile if not ok else ""), flush=True)
    print("Abweichungen: %d   (Ordner: %s)" % (schlecht, ordner))
    return 1 if schlecht else 0


if __name__ == "__main__":
    sys.exit(main())
