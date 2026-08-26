# PyGameZ

**🌐 Sprache / Language:** **🇩🇪 [Deutsch](#-deutsch)** · **🇬🇧 [English](#-english)** · **🇫🇷 [Français](other.readme.md#-francais)** · **🇪🇸 [Español](other.readme.md#-espanol)** · **🇵🇹 [Português](other.readme.md#-portugues)** · **🇵🇱 [Polski](other.readme.md#-polski)** · **🇹🇷 [Türkçe](other.readme.md#-turkce)** · **🇩🇰 [Dansk](other.readme.md#-dansk)** · **🇳🇴 [Norsk](other.readme.md#-norsk)** · **🇸🇪 [Svenska](other.readme.md#-svenska)** · **🇫🇮 [Suomi](other.readme.md#-suomi)** · **🇨🇿 [Čeština](other.readme.md#-cestina)** · **🇸🇮 [Slovenščina](other.readme.md#-slovenscina)** · **🇭🇷 [Hrvatski](other.readme.md#-hrvatski)** <br> Game Available in : German, English, French, Spanish, Portuguese, Polish, Turkish, Danish, Norwegian, Swedish, Finnish, Czech, Slovenian, Croatian (14 languages)

**📋 [Changelog](changelog.md)** (Deutsch / English)

---

<a name="-deutsch"></a>

## 🇩🇪 Deutsch

Eine Desktop-Spielesammlung in Python: **Tkinter** bildet Fenster und Menü,
**Pygame** wird als Spiel-Display in das Tkinter-Fenster eingebettet. Zweiundvierzig
Spiele mit gemeinsamen Optionen, frei belegbarer Steuerung, Highscores,
prozeduralen Soundeffekten und teilweise Mehrspieler-Modus. Die Oberfläche ist
**mehrsprachig** – **14 Sprachen** (Deutsch / English / Français / Español /
Português / Polski / Türkçe / Dansk / Norsk / Svenska / Suomi / Čeština /
Slovenščina / Hrvatski); die Sprache wird beim ersten Start auf einem
**Willkommens-Screen** gewählt, auf dem sich zugleich **Auflösung** und
**Sound** (Standard: aus) einrichten lassen; außer den drei Hauptsprachen
verbergen sich alle weiteren (Spanisch, Portugiesisch und die neun zusätzlichen)
hinter dem Knopf **„Mehr"**. Alles lässt sich später jederzeit in den Optionen
ändern.

### Schnellstart

#### Windows

```bat
install-python.bat    :: einmalig: Python 3.13 + .venv + pygame einrichten
start.bat             :: Spielesammlung starten
```

#### Linux / macOS / Git Bash

```bash
./start.sh            # startet mit .venv, sonst System-python3
```

`start.bat` / `start.sh` verwenden automatisch die virtuelle Umgebung `.venv`,
falls vorhanden, sonst das System-Python. Eine ausführliche Schritt-für-Schritt-
Anleitung steht ganz unten unter **[Installations-Guide](#installations-guide)**.

### Die Spiele

| Spiel        | Modi            | Kurzbeschreibung |
|--------------|-----------------|------------------|
| **Snake**    | 1 / 2 Spieler   | Deluxe-Snake mit 2D- & 3D-Ansicht, Boost, 6 Spielmodi (inkl. Competitive), Goldäpfeln und Prestige |
| **Pong**     | 1 / 2 Spieler   | Klassiker gegen KI oder Spieler 2, umschaltbarer Bewegungsmodus |
| **Air Hockey** | 1 / 2 Spieler | 2D-Physik mit Impulsübertragung, Maussteuerung, KI und Power-Ups |
| **Tic-Tac-Toe** | 1 / 2 Spieler | m,n,k-Spiel auf 3x3 bis 9x9, drei KI-Stärken **oder** lokal X gegen O |
| **Breakout** | 1 Spieler       | Brick-Breaker mit Steinsorten, Power-ups, Combos und vielen Levels |
| **Tetris**   | 1 / 2 Spieler   | Klassik oder Versus (zwei Felder nebeneinander) |
| **Invaders** | 1 Spieler       | Space Invaders: Wellen leeren, Leben schützen |
| **Asteroids** | 1 / 2 Spieler  | Trägheitsphysik, Wellen, UFOs, Power-Ups, Hyperraum - solo oder Koop-Duell |
| **Pac-Man**  | 1 Spieler       | Originalgetreuer Klon: 4 Geister-KIs, Power-Pillen, Tunnel, Früchte, Levels |
| **Flappy Bird** | 1 Spieler    | Schwerkraft-Flug durch Röhren, Münzen, Schild, Tag/Nacht, Medaillen |
| **Doodle Jump** | 1 Spieler    | Auto-Sprung nach oben, Plattform-Typen, Federn, Propeller, Monster |
| **2048**     | 1 Spieler       | Zahlen-Schiebespiel, Ziel: die 2048er-Kachel |
| **Minesweeper** | 1 Spieler    | Der Klassiker mit sicherem Erstklick, Chording, Smiley und Bestzeiten |
| **Sudoku**      | 1 Spieler    | 400 Seed-Level (4 Stufen x 100), 4 Assistenz-Modi mit Punkte-Multiplikator, Notizen, Tipps, 3-Fehler-Limit |
| **Frogger**     | 1 Spieler    | Straße + Fluss + 5 Buchten, Bonus-Fliege, Krokodile, Zeitlimit, 3 Schwierigkeitsgrade |
| **Memory**      | 1 / 2 Spieler | Paare finden auf 4x4 bis 8x6, Flip-Animation, Solo-Wertung oder Duell |
| **Solitär**     | 1 Spieler    | 5 Varianten (Klondike, Spider, FreeCell, Pyramide, TriPeaks) mit Drag & Drop und Undo |
| **Aim Trainer** | 1 Spieler    | Chilliges 3D-Zielschießen: Maus lenkt die Kamera, 4 Modi (Präzision/Reflex/Bewegt/Chill), 3 Themen inkl. schwarzem Loch |
| **Vier gewinnt** | 1 / 2 Spieler | Der Klassiker mit Fall-Animation: 3 KI-Stärken (Minimax) oder lokales Duell |
| **Panzer-Duell** | 1 / 2 Spieler | 2D-Arena-Duell mit Ricochet-Schüssen, Power-Ups, 4 Arenen, KI mit 3 Stärken |
| **Blackjack**    | 1 Spieler    | Casino-Blackjack mit 4-Deck-Schuh, Double/Split, 3:2-Blackjack und dauerhaftem Chip-Konto |
| **Tunnel Racer** | 1 Spieler    | 3D-Neon-Röhrenflug: Endlos-Modus + 30 Level, Tasten- oder Maus-Steuerung, Motion Blur |
| **3D-Labyrinth** | 1 Spieler    | Ego-Raycaster (Wolfenstein-Stil) mit 50 Seed-Leveln, Orbs, Minimap - oder 2D-Draufsicht |
| **Reversi**      | 1 / 2 Spieler | Othello auf 8x8: Steine einschließen und umdrehen, 3 KI-Stärken (Minimax) oder lokales Duell |
| **Kniffel**      | 1 / 2 Spieler | Würfelklassiker mit 13 Kategorien, oberem Bonus und Kniffel; Highscore-Jagd oder 2-Spieler-Hotseat |
| **Wordle**       | 1 Spieler    | Errate das 5-Buchstaben-Wort in 6 Versuchen, Endlos-Streak, farbige Hinweise, 5 Sprachen |
| **T-Rex Runner** | 1 Spieler    | Endloser Wüstenlauf: variabler Sprung, Ducken, Kakteen & Flugsaurier, Tag/Nacht-Wechsel, steigendes Tempo, 3 Schwierigkeitsgrade |
| **Dame**         | 1 / 2 Spieler | 3 Regelwerke wählbar (Deutsche 8×8, Internationale 10×10, Checkers), Schlagzwang & fliegende Dame, 3 KI-Stärken (Minimax) oder lokales Duell |
| **Poker**        | 1 Spieler    | 3 Varianten wählbar: Texas Hold'em gegen KI, 5 Card Draw und Video Poker; Setzrunden, Blinds, dauerhaftes Chip-Konto |
| **Schach**       | 1 / 2 Spieler | Vollständige Regeln (Rochade, En Passant, Umwandlung, Matt/Patt/Remis), 6 KI-Stärken (Minimax + Alpha-Beta) oder lokales Duell, Farbwahl |
| **Mühle**        | 1 / 2 Spieler | Nine Men's Morris mit Setz-/Zieh-/Sprungphase, Mühlen & Schlagen, fliegende Steine abschaltbar, 3 KI-Stärken oder lokales Duell |
| **Simon**        | 1 / 2 Spieler | Senso-Merkspiel: Modi Klassisch/Speed/Reverse/Gemischt + Duell, Ton aus/an/gemischt, 4/6/9 Felder, Bestwert je Modus |
| **Billard**      | 1 / 2 Spieler | 8-Ball, 9-Ball & Übungsmodus in 2D, fester 3D-Ansicht oder frei drehbarer 3D-Kamera; weiche Physik, Zielhilfe, KI mit 3 Stärken |
| **Schiebepuzzle** | 1 Spieler    | 15-Puzzle in 3x3/4x4/5x5: durchnummerierte Kacheln in die Lücke schieben, Klick- oder Pfeilsteuerung, Punkte nach Zügen & Zeit |
| **Mastermind**    | 1 Spieler    | Geheimen Farbcode knacken (3 Modi: 4×6, klassisch, 5×8), schwarze/weiße Rückmeldungs-Pins, Endlos-Streak als Highscore |
| **Bubble Shooter** | 1 Spieler   | Puzzle-Bobble-Klon: gleiche Farben zu Dreiergruppen schießen, Wandreflexion, herabfallende Cluster, 3 Schwierigkeitsgrade |
| **Galgenmännchen** | 1 Spieler   | Wort erraten, bevor der Galgen voll ist; Bildschirmtastatur, Wortlisten je Sprache, 3 Längen-Modi, Endlos-Streak |
| **Block Jump**  | 1 Spieler       | 3D-Jump'n'Run im Minecraft-Stil: texturierte Voxel-Welt, Steve-Figur, Leitern, Zäune & Schleimblöcke, Ego-/Verfolgerkamera, seed-generierte Parkour-Level |
| **Tower Defense** | 1 Spieler     | Endlose Wellen auf 4 Karten abwehren: bis zu 11 Turmtypen mit Ausbau, Verkauf & A/B-Spezialisierung, Bosse, 3 Modi, Aktiv-Fähigkeiten |
| **Minigolf**    | 1 / 2 Spieler   | 360 Bahnen in 40 Kursen (18 handgebaut, 342 seed-erzeugt): Sand, Rampen, Wasser, Gummipuffer, Windmühlen & Wanderblöcke; Scorekarte mit Par und Hole-in-One-Bonus |
| **Pinball**     | 1 / 2 Spieler   | Flipperautomat mit 3 Tischen: Pop-Bumper, Slingshots, Drop-Targets, L-A-M-A-Bahnen, Multiball mit Jackpot, Ball-Save, Nudge & Tilt |
| **Bowling**     | 1 / 2 Spieler   | 10 Frames mit offizieller Strike-/Spare-Wertung, echter Pin-Physik, Hook-Effet und perspektivischer Bahnansicht, 3 Schwierigkeiten |

**Mehrspieler (2 Spieler lokal)** gibt es für **Snake**, **Pong**, **Air Hockey**,
**Tic-Tac-Toe**, **Tetris (Versus)**, **Asteroids (Koop-Duell)**, **Memory (Duell)**,
**Vier gewinnt**, **Panzer-Duell**, **Reversi**, **Kniffel**, **Dame**, **Schach**,
**Mühle**, **Simon (Duell)**, **Billard**, **Minigolf**, **Pinball**
und **Bowling**. Der Modus wird direkt im
Vorspiel-Screen (*Einzelspieler / Mehrspieler*) gewählt.

#### Feature-Details je Spiel

**Snake**
- **NEU - 3D-Ansicht** (Taste **V** im Setup oder Klick auf *Ansicht*): Das
  Spielfeld wird als Echtzeit-3D-Szene gerendert - eine **Verfolgerkamera**
  schwebt hinter der Schlange, gelenkt wird **relativ zur Blickrichtung**
  (links/rechts = drehen, zwei schnelle Drücke = Kehrtwende). Mit Distanz-Nebel,
  Sternenhimmel, Schachbrett-Boden, Banden, rotierenden Futter-Kristallen,
  3D-Partikeln und Kamera-Shake beim Crash; nach dem Game Over umkreist die
  Kamera die Schlange. Beim Boost weitet sich das Sichtfeld. In 3D wählbar:
  *Klassisch* und *Hindernisse* (die Wände sind dort immer fest, 3D gibt es
  nur im Einzelspieler). Die Ansicht wird in `settings.json` gemerkt.
- **NEU - 3D-Kamera-Optionen** (im 3D-Setup die Zeile *3D-Kamera / Smooth-Shake*
  anklicken oder Taste **K**): ein eigenes Menü mit **Smooth-Shake** (sanftere Kamera,
  deutlich weniger Ruckeln beim Bewegen/Drehen), einstellbarem **Sichtfeld (FOV)** und
  **Kamerahöhe** sowie einem Schalter **Ruckeln beim Abbiegen** (Screen-Shake beim
  Links/Rechts-Drehen an/aus). Alles wird in `settings.json` gemerkt.
- **Boost**: Boost-Taste **gedrückt halten** = Turbo (doppeltes Tempo), verbraucht
  Ausdauer (Balken); ist sie leer, schaltet der Boost ab und lädt sich wieder auf.
  Standard P1 = Leertaste/Shift-links, P2 = Enter/Shift-rechts.
- **6 Spielmodi** (im Setup wählbar): *Klassisch*, *Speed-Rush* (wird mit jedem
  Apfel schneller), *Hindernisse* (tödliche Blöcke), *Portale* (Teleporter-Paare),
  *Zeitangriff* (60 Sekunden, so viele Äpfel wie möglich) und *Competitive* (siehe unten).
- **NEU - Competitive** (Einzelspieler): Endlos-Modus mit **Level-Aufstieg** - man
  startet mit genau **einem** Apfel und kann anfangs nicht mehr bekommen; je mehr
  Äpfel man insgesamt sammelt, desto höher das **Level**, das laufend einen weiteren
  Apfel gleichzeitig aufs Feld legt und den Punkte-Multiplikator erhöht.
  **Blaue Äpfel** öffnen eine **Slot-Machine**: eingesetzt wird die Länge, das
  Walzen-Ergebnis vervielfacht bzw. verkleinert den Einsatz und lässt für kurze Zeit
  **zusätzliche Äpfel** spawnen (Jackpot bei drei gleichen Symbolen).
  **Lila Äpfel** (Gambling) setzen einen Anteil der **Größe** aufs Spiel und
  multiplizieren diesen Teil zufällig, der Rest bleibt sicher (neue Größe =
  Größe·(1-p) + Größe·p·Faktor): **normal** fix 50 % Einsatz mit **x0.5 .. x1.5**,
  im **HARDCORE** riskanter mit **75-90 %** Einsatz und **x0.25 .. x2.25**. Die
  **Größe** steht als **Kommazahl oben links** und wird exakt weitergeführt, sodass
  weitere Wetten darauf aufbauen. Es gibt **15 Level** (Multiplikator bis x16, bis zu 16 Äpfel gleichzeitig);
  die Stufen stehen in `games/levels/snake-comp.json` und lassen sich dort ohne
  Code-Änderung erweitern, die übrige Feineinstellung in `competitive.py`.
- **NEU - HARDCORE** (Schalter im Competitive-Setup, Taste **H**): jeder **Boost
  frisst die Länge** deiner Schlange; ein rot leuchtender **HARDCORE-Schriftzug**
  markiert den Modus. Nur im Competitive verfügbar; die Länge fällt nie unter das
  Minimum. Wird in `settings.json` gemerkt.
- **Goldäpfel** (zeitweise) geben viele Punkte und füllen den Boost sofort auf.
- Optionale **Wände-durchgehen**, Bonus-Äpfel, **Prestige** (Einzelspieler, Taste **P**).
- **NEU - Personalisieren** (Pinsel-Knopf ganz oben rechts im Setup, oder Taste **C**):
  ein reines Optik-Menü ("Mods", die *nie* das Spiel verändern) mit zwei Reitern:
  - **Kopf**: die **Kopffarbe** der Schlange - 4 Blau-Türkis-Vorlagen (von mehr Blau
    bis mehr Türkis), Rot, Orange und eine **eigene Farbe** über RGB-Regler.
  - **Raster (Wegweiser)**: blendet ein **Koordinaten-Raster** über das Feld -
    **Reihen-Nummern** (am linken und rechten Rand) und **Spalten-Buchstaben**
    (oben/unten). So sieht man auf grossen Feldern sofort, dass z.B. der
    Apfel bei *8a* in derselben Reihe *8* liegt wie die eigene Position *8z*. Die
    Farbreihenfolge (5 Vorlagen + zwei eigene Farben A/B) bestimmt das Farbthema.
  - **Banner**: das Multiplikator-Banner (z.B. beim lila Apfel) **an-/ausschalten**
    sowie **Größe** (kleiner/größer) und **Deckkraft** (transparenter) einstellen -
    mit Live-Vorschau.
  Alles wird in `mem-ngb.json` gespeichert; die visuelle Personalisierung läuft über
  das Modul `ngb.py`.
- Optik: abgerundete Schlange mit Augen (Kopf standardmäßig türkis), Boost-Glow, Partikel.

**Pong**
- Einzelspieler gegen KI, Mehrspieler = Spieler 2 rechts. Bis 5 Punkte.
- **Bewegungsmodus je Steuerung umschaltbar**: *Dauer* (einmal drücken -> fährt
  weiter, Standard) oder *Halten* (bewegt nur solange gedrückt).
  Umschalten: **X** = Steuerung 1, **N** = Steuerung 2 (wird in `settings.json` gemerkt).
- Ball-Physik mit Beschleunigung und Winkel je nach Treffpunkt.

**Air Hockey**
- **Echte 2D-Physik**: runde Schläger und Puck mit Impulsübertragung - der Puck
  übernimmt die Schlägergeschwindigkeit beim Treffer; Banden mit Restitution,
  leichte Eisreibung, Tore als Öffnungen in den Seitenwänden.
- **Maussteuerung** im Einzelspieler: der Schläger folgt der Maus (jede Taste
  schaltet zurück auf Tastensteuerung). Tastatur: Richtungstasten in 8 Richtungen,
  Mehrspieler = P1 links (WASD), P2 rechts (IJKL).
- **KI mit drei Stärken** (Leicht/Mittel/Schwer): verteidigt das eigene Tor,
  greift in der eigenen Hälfte an und umkurvt den Puck gegen Eigentore.
- **Power-Ups** (abschaltbar): *XL* (größerer Schläger), *TOR* (Gegnertor
  schrumpft), *>>* (schnellerer Schläger) - sie gehören dem Spieler, der den
  Puck zuletzt berührt hat.
- Setup: Schwierigkeit, **Tore bis zum Sieg** (3/5/7/10), Power-Ups an/aus
  (gespeichert in `settings.json`). Nach jedem Tor Anspiel beim Gegentor-Nehmer.
- Optik: Puck-Leuchtspur, Partikel, pulsierende Tor-Mäuler, Effekt-Anzeigen.

**Tic-Tac-Toe**
- Setup: Schwierigkeit (Easy/Medium/Hard) und Brettgröße 3x3..9x9; Gewinnlänge
  K = 3 (3x3), 4 (4x4), sonst 5.
- **1 Spieler** gegen KI (Hard auf 3x3 ist unschlagbar) **oder 2 Spieler** lokal
  (X gegen O, per Klick abwechselnd). Bei Game Over: Enter/Klick = neue Runde,
  **S** = Einstellungen.

**Breakout**
- Steinsorten: Normal, **Stahl** (unzerstörbar), **Bombe** (explodiert), **Gold**
  (Extrapunkte).
- Power-ups: Laser, Feuerball, Klebrig, Schild, Münze u.a.; **Combo-Multiplikator**.
- Effekte: Partikel, Ball-Spuren, Screen-Shake, Punkte-Popups, viele Level-Muster.
- Setup: **1/2/3** = Schwierigkeit, **Links/Rechts** = Ballfarbe, **Hoch/Runter** =
  Startlevel, **M** = Aufbau. Spiel: Maus/Pfeile, **Leertaste** startet den Ball
  (feuert Laser), **P/Esc** = Pause.

**Tetris**
- Links/Rechts verschieben, Hoch = drehen, Runter = Soft-Drop, Aktion = Hard-Drop.
- Volle Reihen geben Punkte, alle 10 Reihen steigt das Level.
- **Versus**: Wessen Stapel zuerst oben anstößt, verliert.

**Invaders** – zwei Modi (im Vorspiel wählbar):
- **Klassik**: klassischer Alien-Block; danach im Setup-Screen wählbar:
  **Bewegung** (nur links/rechts *oder* frei mit WASD) und **Zielen** (immer nach
  oben *oder* zur **Maus** – dann schießt man dorthin, wo der Mauszeiger ist).
  Zerstörte Aliens lassen manchmal Power-ups fallen.
- **Arena (frei)**: freie Bewegung in alle Richtungen, Gegner strömen von allen
  Rändern; man zielt in Bewegungsrichtung, Waffe mit **1–4** wechseln.
Gemeinsam: Levelsystem mit **Boss** in jedem 4. Level, vier Waffen (Blaster,
Streuschuss, Schnellfeuer, Laser), Power-ups (Extraleben, Schild, Waffen-Upgrade),
Explosions-Effekte, Highscore.

**Asteroids**
- **Trägheitsphysik**: Hoch = Schub in Blickrichtung, Links/Rechts = drehen,
  das Schiff driftet weiter (leichte Dämpfung); alles wickelt über die
  Bildschirmränder. Klassische **Vektor-Optik** mit Schubflamme und
  Sternenhimmel; jeder Brocken hat eine eigene zufällige Polygon-Form.
- Brocken zerspringen in zwei kleinere (3 Größen, **20/50/100 Punkte**),
  **Wellen** mit steigender Anzahl und Banner-Einblendung.
- **UFO** (abschaltbar): kreuzt regelmäßig den Bildschirm und zielt auf die
  Schiffe (Zielfehler je Schwierigkeit) - 200 Punkte für den Abschuss.
- **Power-Ups** (abschaltbar), fallen aus zerstörten Brocken: **S**child (6 s
  unverwundbar), **T** = Dreifachschuss, **R** = Schnellfeuer.
- **Hyperraum** (Runter-Taste): Nottransport an eine Zufallsposition mit 4 s
  Abklingzeit - und 12 % Risiko, dabei zu zerschellen.
- 3 Leben, sicheres Respawnen mit Unverwundbarkeits-Blinken, **Extraleben
  alle 5000 Punkte**; Explosions-Partikel und Kamera-Shake.
- **Koop-Duell** (Mehrspieler): beide Schiffe fliegen gleichzeitig, getrennte
  Leben und Punkte - wer mehr Punkte hat, gewinnt.
- Setup: Schwierigkeit, UFOs an/aus, Power-Ups an/aus (in `settings.json`).

**Pac-Man**
- **Klassisches 28x31-Labyrinth** im Neon-Look mit Pillen, 4 Power-Pillen,
  Tunnel-Warp an den Seiten und Geisterhaus in der Mitte.
- **Vier Geister mit den Original-Verhaltensweisen** (Ziel-Kachel-KI):
  *Blinky* jagt direkt, *Pinky* legt sich in den Hinterhalt (4 Kacheln voraus),
  *Inky* nutzt einen Vektor über Blinky, *Clyde* weicht aus der Nähe aus.
- **Scatter/Chase-Phasen** im Wechsel (Geister drehen bei jedem Wechsel um);
  **Power-Pille** macht Geister blau und essbar (Kette 200/400/800/1600),
  danach kehren die Augen ins Haus zurück.
- Geisterhaus mit **gestaffelter Freigabe**, **Früchte** als Bonus (je Level),
  **3 Leben**, **Extraleben bei 10.000**, Level-System (wird schneller),
  Death-Animation, READY-/GAME-OVER-Screens.
- Setup: **Schwierigkeit** (Normal/Schwer/Extrem) – Geistertempo & Frightened-Zeit.
- Steuerung: **Pfeile oder WASD**.  Enter = neu, S = Setup.

**Flappy Bird**
- **Schwerkraft-Physik**: Leertaste / Pfeil hoch / W / **Mausklick** lässt den
  Vogel flattern; er neigt sich je nach Steig-/Sinktempo.
- Endlose **Röhrenpaare** mit Lücke (+1 pro Röhre); **Münzen** (Bonus) und
  **Schild**-Power-up (überlebt eine Kollision) erscheinen in den Lücken.
- **Tag/Nacht-Themen** wechseln mit der Punktzahl; driftende Wolken (Parallax),
  scrollender Boden.
- Schwierigkeit (Leicht/Normal/Schwer): Lückengröße, Tempo, Röhrenabstand –
  die Lücke wird mit steigender Punktzahl etwas enger.
- **Medaillen** (Bronze/Silber/Gold/Platin) nach dem Game Over, Crash-Animation
  mit Kamera-Shake, Highscore.

**Doodle Jump**
- Der Doodler **springt automatisch** beim Landen; gesteuert wird nur
  links/rechts (mit Trägheit), die Ränder sind offen (**Wrap-around**); die
  Kamera scrollt mit dem Aufstieg.
- **Plattform-Typen**: grün (normal), blau (beweglich), braun (zerbricht),
  weiß (verschwindet). **Sprungfedern** geben einen Superhüpfer, der
  **Propeller-Hut** trägt kurz automatisch nach oben (und macht unverwundbar).
- **Monster**: Berührung ist tödlich – man kann sie aber mit Pfeil hoch /
  Leertaste **abschießen** (Extrapunkte).
- Punkte = erreichte Höhe; Schwierigkeit steigt mit der Höhe. Highscore.
- Steuerung: links/rechts = bewegen, Pfeil hoch / Leertaste = schießen.

**2048** – Pfeile/WASD schieben alle Kacheln; gleiche Zahlen verschmelzen.

**Minesweeper**
- Drei Stufen: **Einsteiger** (9x9, 10 Minen), **Fortgeschritten** (16x16, 40),
  **Experte** (30x16, 99) - die **Bestzeit je Stufe** wird gespeichert und im
  Setup angezeigt.
- Der **erste Klick ist immer sicher** (Minen werden erst danach verteilt,
  das 3x3-Feld um den Klick bleibt frei).
- **Linksklick** = aufdecken, **Rechtsklick** = Flagge (optional mit
  Fragezeichen-Zyklus), **F** = Flagge unter dem Mauszeiger, **R** = neu.
- **Chording**: Klick auf eine fertige Zahl deckt die restlichen Nachbarn auf.
- Klassisches HUD: Minenzähler, **klickbarer Smiley** (staunt/Sonnenbrille/tot),
  Timer; falsche Flaggen werden am Ende durchgestrichen, Konfetti beim Sieg.
- Punkte = Grundwert der Stufe minus Sekunden.

**Sudoku**
- **400 Level**: 4 Schwierigkeitsgrade (Leicht/Normal/Schwer/Experte) x 100
  Level. Die Puzzles sind **Seed-generiert und eindeutig lösbar** - Level 12
  von "Schwer" ist auf jedem PC dasselbe Puzzle. Gelöste Level werden
  gespeichert und in der Levelauswahl abgehakt.
- **4 Spielmodi** (Auswahl vor dem Start) mit Punkte-Multiplikator:
  **Klassisch** (x2,0 - keine Hilfen), **Notizen** (x1,5 - + Bleistift-
  Notizen), **Komfort** (x1,0 - + Fehler rot, Konflikt- und Gleiche-Ziffer-
  Hervorhebung, korrekte Eingaben rasten ein), **Assistent** (x0,7 - + Tipp-
  Funktion, max. 3).
- Jede Eingabe wird sofort gegen die Lösung geprüft; mit aktivem
  **3-Fehler-Limit** (Option im Setup) ist beim dritten Fehler Schluss.
- Steuerung: Pfeile/WASD = Zelle, **1-9** = Ziffer (auch Ziffernblock),
  **0/Backspace/Rechtsklick** = radieren, **N** = Notizen, **H** = Tipp,
  **R** = Level neu, **Q** = Levelwahl; komplett mit der Maus spielbar
  (Ziffernfeld rechts). Nach Spielende blendet **A** den Banner aus und
  zeigt die komplette **Lösung** auf dem Brett (nochmal A = zurück).
- Punkte = (Basis der Stufe - Zeit - Fehler - Tipps) x Modus-Multiplikator.

**Frogger**
- 5 Fahrspuren (Autos/Laster) und 5 Flussbahnen (Stämme, Schildkröten, die ab
  höheren Leveln **abtauchen**); oben 5 Ziel-Buchten - alle füllen = nächstes
  Level, alles wird schneller.
- Extras: **Bonus-Fliege** (+200) in leeren Buchten, **Krokodile** besetzen ab
  höheren Leveln Buchten, **Zeitlimit-Balken** je Frosch, Extraleben bei 10 000.
- 3 Schwierigkeitsgrade (Tempo, Verkehrsdichte, Zeit); Punkte je neuer Reihe,
  Bucht = 50 + Zeitbonus, Level komplett = +1000.

**Memory**
- Brettgrößen **4x4, 6x6, 8x6**; Motive aus Form-Farb-Kombinationen, komplett
  mit Primitiven gezeichnet; **Flip-Animation**, Fehlpaare klappen automatisch
  zurück.
- **Solo**: Basis - 15 je Zug - 2 je Sekunde (mind. 100). **Duell** (lokal):
  abwechselnd, Treffer = nochmal dran, Sieger = meiste Paare.

**Solitär**
- **5 Varianten** im Vorspiel-Screen: Klondike (Ziehen 1/3 als Option), Spider
  (1/2/4 Farben), FreeCell (Supermove-Limit), Pyramide (13er-Paare, 2 Redeals)
  und TriPeaks (±1-Kette mit Combo-Multiplikator).
- **Drag & Drop** oder Klick-Klick, **Rechtsklick** = aufs Fundament,
  **U** = unbegrenztes Undo, **R** = neues Blatt, Leertaste = Stock.
- Karten werden ohne Bild-Dateien gerendert (`games/cards.py`); alle Varianten
  teilen sich eine Highscore-Liste mit variantenspezifischen Formeln.

**Aim Trainer**
- **Echtes Software-3D** (wie Snakes 3D-Modus): festes Fadenkreuz in der
  Bildmitte, **direkte 1:1-Maussteuerung wie im Shooter** (Pointer-Capture:
  der Cursor wird im Fenster eingefangen, Esc gibt ihn frei; einstellbare
  Empfindlichkeit, Yaw unbegrenzt, Pitch ±60°). Linksklick schießt exakt
  durch die Mitte, mit Mündungsblitz, Tracer und Treffer-Partikeln.
- **4 Modi**: Präzision (60 s, 3 Kugeln, Genauigkeits-Bonus), Reflex
  (30 Ziele einzeln, Reaktionszeit-Statistik), Bewegte Ziele (Bahnen +
  Combo-Multiplikator bis x4) und Chill (endlos, ohne Strafe, **E** beendet).
- **3 Themen** (im Setup, gespeichert): **Weltraum** mit Sternenkugel,
  **schwarzem Loch mit leuchtendem Ring** und Planet (Standard), Neon-Arena
  mit Boden-Grid und Synthwave-Sonne, sowie Schießstand-Halle.
- Empfindlichkeit auch mitten im Spiel per **+/-** änderbar; dazu ein
  **einstellbarer Motion Blur** (0-80 %) für extra Chill-Optik - beides
  wird gespeichert.

**Vier gewinnt**
- 7x6-Brett mit **Fall-Animation**, Hover-Vorschau und pulsierender Sieg-Linie;
  Maus, Pfeiltasten oder Direktwahl **1-7**.
- **3 KI-Stärken** (Minimax mit Alpha-Beta-Suche): Leicht übersieht bewusst
  Drohungen, Mittel blockt zuverlässig, Schwer plant tief voraus - oder
  **2 Spieler** lokal am selben Gerät.
- Nach jeder Runde wechselt der Startspieler; der Highscore zählt die
  **Siege gegen die KI** einer Sitzung.

**Panzer-Duell**
- 2D-Arena-Duell: **Schüsse prallen einmal von Wänden ab** (Ricochet) - auch
  um die Ecke treffen (oder sich selbst!). First-to-5-Runden mit Countdown.
- **4 Arenen** (Offen, Kreuz, Säulen, Labyrinth) oder zufällige Rotation;
  **Power-Ups**: Schnellfeuer, Schild, Dreifach-Schuss.
- **KI mit 3 Stärken** - die schwere zielt mit Vorhalt und schießt absichtlich
  über die Bande - oder **2 Spieler** an einer Tastatur (P1 WASD+Leertaste,
  P2 Pfeile+Enter).

**Blackjack**
- Echte Casino-Regeln: **4-Deck-Schuh**, Dealer steht auf 17, **Blackjack
  zahlt 3:2**, Dealer-Peek bei Ass/10; **Verdoppeln** und **einmal Teilen**
  (geteilte Asse bekommen je eine Karte).
- **Dauerhaftes Chip-Konto**: Start mit 500, Stand und **Rekord** überleben
  jeden Neustart (`mem.json`); unter 10 Chips gibt es 500 frische - der
  Rekord bleibt.
- Bedienung per Chip-Buttons und Tasten (**H**it/**S**tand/**D**ouble/Split
  **X**, **1-4** = Einsatz, Enter = Geben) mit Karten-Animationen und
  Hole-Card-Flip.

**Tunnel Racer**
- **3D-Neon-Röhrenflug** (Software-Renderer wie beim Aim Trainer): Balken,
  Blöcke und **Ring-Blenden zum Durchfädeln**, Münzen auf der Ideallinie.
- **Zwei Modi**: Endlos (Tempo steigt bis Cap, Highscore) und **30 Seed-Level**
  mit Ziel, Zeitbonus und abgehaktem Fortschritt.
- **Tasten-Steuerung** (Standard) oder **direkte Maus-Steuerung**
  (Pointer-Capture, Taste **C**); dazu einstellbarer **Motion Blur** (Taste
  **B**, 0-80 %) - alles wird gespeichert.

**3D-Labyrinth**
- **Ego-Raycaster im Wolfenstein-Stil** (DDA, Distanz-Nebel, Sprites) mit
  Mouselook + WASD, **Minimap** (Taste **M**) und grün pulsierendem Ausgang -
  alternativ eine klassische **2D-Draufsicht** (Taste **V** im Setup).
- **50 Seed-Level**, die stetig wachsen; der Ausgang liegt immer am
  entferntesten Punkt, **Orbs** auf dem Weg geben Bonuspunkte.
- Punkte: 500 je Level + 100 je Orb + Zeitbonus; gelöste Level werden
  abgehakt, die Sitzung summiert sich zum Highscore.

**Reversi**
- **Othello auf 8x8**: Steine setzen, die gegnerische Reihen einschließen, und
  alle eingeschlossenen umdrehen; ungültige Züge sind gesperrt, bei fehlendem
  Zug wird **automatisch gepasst**.
- **Einzelspieler gegen die KI** (3 Stärken: Negamax mit Alpha-Beta,
  Positionsgewichtung + Mobilität) **oder lokales Duell** Schwarz gegen Weiss.
- Gültige Felder werden markiert; Steuerung per **Maus** oder Auswahlrahmen
  (Pfeile + Leertaste/Enter). Jeder Sieg gegen die KI zählt einen Punkt für den
  Highscore.

**Kniffel**
- **Würfelklassiker**: 5 Würfel, bis zu 3 Würfe je Zug, Würfel einzeln
  **halten**; danach eine der **13 Kategorien** buchen (mit Live-Vorschau der
  möglichen Punkte).
- Kompletter Wertungsblock: oberer Block mit **63er-Bonus (+35)**, Dreier-/
  Viererpasch, Full House, kleine/große Straße, **Kniffel (50)** und Chance.
- **Einzelspieler als Highscore-Jagd** auf die höchste Endsumme oder
  **2-Spieler-Hotseat** mit zwei Blöcken nebeneinander; Bedienung per Maus oder
  Tasten (Leertaste, 1-5, Pfeile, Enter).

**Wordle**
- Errate das **5-Buchstaben-Wort in 6 Versuchen**; farbige Rückmeldung
  (grün/gelb/grau) mit korrekter **Doppelbuchstaben-Zählung** und mitfärbender
  Bildschirmtastatur.
- **Endlos-Streak**: jedes gelöste Wort bringt Punkte (weniger Versuche = mehr),
  das erste ungelöste Wort beendet die Partie - Summe = Highscore.
- **Wortlisten je Sprache** (nur A-Z); Rateversuche werden nicht gegen ein
  Wörterbuch geprüft. Eingabe per Tastatur oder anklickbarer Bildschirmtastatur.

**Schach**
- **Vollständiges Schach**: alle Figurenzüge inkl. **Rochade**, **En Passant**
  und **Bauernumwandlung** (Figur wählbar); **Schach, Schachmatt und Patt** sowie
  Remis durch **50-Züge-Regel**, **dreifache Stellungswiederholung** und
  **ungenügendes Material**.
- **6 KI-Stärken** von *Anfänger* bis *Meister* (Negamax mit Alpha-Beta,
  Zugsortierung, Figur-/Feldwert-Tabellen, Ruhesuche); ein **Zeitbudget** hält
  jeden KI-Zug flüssig. **Farbwahl** Weiß/Schwarz oder **lokales Duell**.
- Legale Züge werden markiert, Schach hervorgehoben; Steuerung per **Maus** oder
  Auswahlrahmen. Jeder Sieg gegen die KI zählt einen Highscore-Punkt.

**Mühle**
- **Nine Men's Morris** mit allen drei Phasen: **Setzen** (je 9 Steine),
  **Ziehen** entlang der Linien und **Springen** bei nur noch 3 Steinen (per
  Schalter abschaltbar).
- Geschlossene **Mühlen** entfernen einen Gegnerstein (bevorzugt außerhalb
  gegnerischer Mühlen); verloren, wer unter 3 Steine fällt oder zugunfähig ist.
- **3 KI-Stärken** (Minimax mit Alpha-Beta, phasengerechte Bewertung) oder
  **lokales Duell**; mit Zughinweisen, Mühlen-Hervorhebung und Steinzähler.

**Simon**
- **Senso-Merkspiel**: die leuchtende Folge wächst jede Runde und muss exakt
  nachgetippt werden.
- **Modi**: *Klassisch*, *Speed* (wird schneller), *Reverse* (rückwärts),
  *Gemischt* (Modus wechselt je Runde) und **Duell** zu zweit (abwechselnd
  anhängen & wiederholen).
- **Ton** *aus / an / gemischt* (im gemischten Ton trainierst du visuell UND
  akustisch), **4/6/9 Felder** als Schwierigkeit; **Bestwert je Modus** wird
  gespeichert. Steuerung per Maus oder Tasten 1-9.

**Billard**
- **8-Ball**, **9-Ball** und ein regelfreier **Übungsmodus**, gegen die KI (mit
  Zielhilfe) oder **zu zweit lokal**.
- **Drei frei wählbare Ansichten**: klassische **2D-Draufsicht**, feste
  **3D-Schrägperspektive** mit schattierten Kugeln und **frei drehbare
  3D-Kamera** (rechte Maustaste). Alle Bewegungen sind zeitschritt-basiert und
  **weich abgebremst** (Reibung, Teilschritte gegen Durchtunneln).
- **Stoßen**: linke Maustaste gedrückt halten lädt die Kraft, Loslassen stößt;
  Ziellinie und Kraft-Meter helfen. Nach Foul **Ball in Hand**. Die Ansicht (V)
  wird in `settings.json` gemerkt; gewonnene Frames zählen für den Highscore.

**Schiebepuzzle**
- 15-Puzzle in drei Größen: **3×3** (leicht), **4×4** (klassisch) und **5×5**
  (schwer); durchnummerierte Kacheln in die freie Lücke schieben.
- Immer lösbar (Mischen über viele zufällige Züge). Steuerung per **Klick** auf
  eine Kachel in Reihe/Spalte der Lücke (die ganze Linie rutscht) oder
  **Pfeiltasten**.
- Punkte = Grundwert je Größe minus Züge und Zeit; nach dem Lösen geht es sofort
  mit einem neuen Feld weiter.

**Mastermind**
- Verdeckten **Farbcode** knacken; nach jedem Tipp gibt es **schwarze** Pins
  (richtige Farbe + Position) und **weiße** Pins (richtige Farbe, falsche Stelle).
- **3 Modi**: Leicht (4 Stifte / 6 Farben / 12 Reihen), Klassik (4/6/10) und
  Schwer (5 Stifte / 8 Farben); Farben dürfen mehrfach vorkommen.
- Bedienung per Farbpalette (Klick oder Tasten **1–8**), OK/Enter wertet die
  Reihe aus. **Endlos-Streak** wie bei Wordle: jeder geknackte Code bringt Punkte.

**Bubble Shooter**
- **Puzzle Bobble** auf einem Wabenraster: mit der Maus zielen, Kugeln nach oben
  schießen, ab **drei gleichen Farben** platzt die Gruppe.
- Kugeln, die den Halt zur Decke verlieren, **fallen** herab (Bonus); Schüsse
  **prallen an den Wänden ab**, mit Vorschau der nächsten Kugel.
- **3 Modi** (4/5/6 Farben, teils nachrückende Reihen); Game Over an der roten
  Linie.

**Galgenmännchen**
- Wort **Buchstabe für Buchstabe** erraten; jeder Fehler zeichnet ein Teil des
  Galgenmännchens, nach **6 Fehlern** verloren.
- **Wortlisten je Sprache** (nur A–Z), **3 Längen-Modi** (kurz / gemischt / lang);
  Eingabe per Tastatur oder anklickbarer Bildschirmtastatur.
- **Endlos-Streak**: jedes erratene Wort bringt Punkte (mehr Restleben +
  längeres Wort = mehr).

**Block Jump**
- **3D-Jump'n'Run im Minecraft-Stil** (Software-3D wie Snakes 3D-Modus): springe
  über eine schwebende **Voxel-Welt** aus Blöcken bis zum leuchtenden Ziel.
- **Minecraft-Skin**: alle Blöcke tragen echte **Pixeltexturen** (Gras mit
  grünem Überhang, Erde, Stein, Eichenbretter, Diamant, Schleim, Holz), die
  perspektivisch korrekt in Texel-Flächen zerlegt werden – der Detailgrad
  richtet sich nach dem Abstand (**T** schaltet hoch / niedrig / aus).
- Dazu im gleichen Stil: **Steve-Figur** mit Gesicht, Armen, Beinen und
  Laufanimation (Verfolgerkamera), die **Hand im Ego-Modus**, **Beacon-Strahl**
  am Ziel, rotierende **Gold-Barren** als Coins, quadratische **Sonne**,
  driftende **Pixel-Wolken** und ein HUD mit **Herzen**.
- Blocktypen: feste Blöcke (Gras/Erde/Stein/Holz), **Leitern** (klettern),
  **Zäune** (überspringen), **Schleimblöcke** (katapultieren) und **Coins**.
- Kamera **standardmäßig 1st-Person wie Minecraft**, **V** schaltet auf die
  Verfolgerkamera; **Maus-Look** mit Pointer-Capture, einstellbarer **Motion-Blur**
  (**B**) und Empfindlichkeit (**+/-**).
- **Seed-generierte Parkour-Level** werden schwerer; Ziel = Punkte + Zeitbonus,
  Coins +50, ein Absturz kostet ein Leben (Start mit 3). Steuerung: WASD/Pfeile,
  **Leertaste** springen.

**Tower Defense**
- **Endlos-Wellenabwehr** auf **4 Karten** (Wiese, Schlucht, Kreuzung,
  Spießrutenlauf) mit eigenem Pfad; gesperrte Karten schaltet die beste
  erreichte Welle frei, alle **8 Wellen** marschiert ein **Boss**.
- **3 Modi**: Klassisch (7 Türme, Hauptmodus), Kompakt (4 Türme, 2 Stufen)
  und Maximal (**11 Türme**, **A/B-Spezialisierung** auf höchster Stufe,
  Spezialgegner, Aktiv-Fähigkeiten **Meteor/Frostnova/Goldsegen**).
- **11 Turmtypen** von Pfeil bis Laser & Goldbank, je bis zu **3 Ausbaustufen**,
  Verkauf mit 70% Erstattung; Gegner mit Panzerung, Regeneration, Teilung,
  Tarnung, Heil-Aura und eigener Flug-Route.
- **Ökonomie**: Gold je Abschuss, Wellen-Bonus + 5% Zinsen; Punkte je Abschuss
  und Welle. **F** = Tempo x2, **G** = Reichweiten, Rechtsklick bricht ab.

**Minigolf**
- **360 Bahnen in 40 Kursen**: *Classic* und *Pro* mit je 9 handgebauten Bahnen,
  die **Tour** mit 38 Kursen zu je 9 seed-erzeugten Bahnen (342 Stück) bei
  steigender Schwierigkeit, dazu *Random* aus allem zusammen. Kurs 7, Bahn 3
  sieht überall gleich aus - gespeichert werden muss dafür nichts.
- **Untergründe & Hindernisse**: Sand bremst, Rampen beschleunigen, Wasser
  kostet einen Strafschlag, Gummipuffer geben Tempo zurück, Windmühlen und
  Wanderblöcke verlangen Timing. Die Physik läuft wie beim Billard in
  Teilschritten mit Reibung - nichts ruckt, nichts tunnelt durch die Bande.
- **Steuerung**: Maus zielt, linke Maustaste halten lädt die Kraft, Loslassen
  schlägt (alternativ Pfeile + Leertaste). **R** bricht einen geladenen Schlag
  ab, ohne zu putten. **G** blendet die Ziellinie um, **P** das Aufnehmen.
- **Scorekarte** rechts mit Par und Schlägen je Bahn; zu zweit spielt jeder
  dieselbe Bahn nacheinander. Punkte: 600 je Bahn, ±300 je Schlag unter/über
  Par, **500 extra für ein Hole-in-One**. Die niedrigste Schlagzahl je Kurs
  steht im Abschnitt `minigolf` von `mem.json`.
- **Aufnehmen ist abschaltbar**: Standardmäßig endet eine Bahn nach acht
  Schlägen und wird mit dem Mindestwert gewertet. Wer lieber bis zum Einlochen
  weiterspielt, stellt *Aufnehmen* im Setup auf AUS (oder drückt **P**).

**Pinball**
- **Drei Tische**: *Classic* (drei Pop-Bumper, eine Target-Bank), *Space* (vier
  Bumper im Karo, zwei Banks) und *Lama* (offenes Feld, sechs Targets im Bogen);
  3 oder 5 Bälle je Partie, zu zweit im Wechsel Ball für Ball.
- **Alles, was ein Flipper braucht**: Schussbahn mit Ladebalken (zu schwach
  gezogen? der Ball rollt zurück und darf noch einmal), zwei Flipper,
  Slingshots, Drop-Target-Banks, vier **L-A-M-A**-Rollover-Bahnen, Saucer mit
  Ball-Lock, **Multiball samt Jackpot**, sechs Sekunden **Ball-Save**, Nudge
  über die Hoch-Taste und **TILT** bei dreimal zu hastigem Anstoßen.
- **Multiplikator bis x5** über geräumte Target-Banks und komplette Bahnen;
  Bumper zählen 100, Slingshots 50, Targets 250 - im Multiball zahlen die
  Bumper 2500 als Jackpot.
- Flipper links/rechts über die belegten Tasten (zusätzlich Shift links/rechts)
  oder per Maus (linke/rechte Bildhälfte). Der Bestwert je Tisch liegt im
  Abschnitt `pinball` von `mem.json`.

**Bowling**
- **Zehn Frames nach offiziellen Regeln** samt Strike-, Spare- und Bonuswürfen
  im zehnten Frame (Maximum: die 300). Die **Scorecard** unter der Kopfzeile
  zeigt alle Frames mit X, / und laufender Summe.
- **Wurf in vier Schritten**: Position, Zielwinkel, Effet und Kraft. Jeder
  Regler pendelt von allein und wird mit der Aktionstaste festgelegt - oder mit
  Links/Rechts von Hand eingestellt, dann hält das Pendeln an.
- **Echte Pin-Physik**: zehn Pins als Kreise mit Masse, die sich gegenseitig
  umwerfen - ein Strike ist Ergebnis der Physik, nicht des Zufalls. Die Bahn ist
  vorn geölt, der **Hook** greift erst im hinteren Drittel und zieht den Ball in
  die Pocket.
- Perspektivische Bahnansicht mit Rinnen, Zielpfeilen und Pin-Deck; drei
  Schwierigkeiten (*Leicht/Normal/Pro*) verändern Pendeltempo und Streuung.
  Der Bestwert je Stufe liegt im Abschnitt `bowling` von `mem.json`.

Highscores werden im Abschnitt `highscores` von `mem.json` (neben dem Code)
gespeichert – gemeinsam mit der Sprache (Abschnitt `mem`).

### Die Oberfläche

Die komplette Oberfläche ist selbst gezeichnet (reines Tkinter + Pygame, keine
Zusatzpakete) und auf einen modernen Launcher-Look getrimmt:

- **Sidebar mit Spieleliste**: jede Zeile hat ein eigenes **Mini-Piktogramm** in
  der Akzentfarbe des Spiels, zeigt den aktuellen **Highscore (★)** und reagiert
  mit weich animierten Hover-Effekten. Das laufende Spiel bleibt farbig markiert;
  bei kleinen Fenstern **scrollt** die Liste per Mausrad.
- **Status-Karte** unten links mit **Zustands-LED** (grau = Menü, grün = läuft,
  gold = Pause, rot = Game Over) und **Live-FPS-Anzeige**.
- **Startbildschirm** mit Aurora-Lichtern, Parallax-Sternenfeld samt
  Sternschnuppen, schwebendem Logo mit Orbit-Funken, einem **klickbaren
  Spiele-Raster** direkt unter dem Logo (alle Spiele mit Hover-Effekt in
  ihrer Akzentfarbe) und einem **Highscore-Laufband**.
- **Effekte überall**: weiche Screen-Übergänge beim Wechseln, Funken beim
  Bestätigen im Menü, **Konfetti-Regen bei neuem Highscore** und ein echter
  **Weichzeichner** hinter dem Pause-Overlay.
- Der **Vorspiel-Screen** jedes Spiels erscheint in dessen Akzentfarbe und zeigt
  den bisherigen Rekord als Chip an.
- **Einheitlicher Spiel-Look**: alle 42 Spiele nutzen dieselbe Theme-Palette und
  -Schrift wie das Menü - HUDs, Setup-Screens und Overlays folgen dem in den
  Optionen gewählten Design (v4.1 / v4 / Classic), die Spielfelder behalten ihre
  Identitätsfarben. Auflösungswechsel mitten im Spiel übernimmt jedes Spiel
  sauber, und die Spielnamen im Menü sind sprachabhängig
  (z. B. „Schach" → "Chess" / « Échecs »).
- **In-Game-Wiki** („LamaWiki"): ausführliche Hilfe zu jedem Spiel (Steuerung,
  Modi, Punkte, Tipps) plus allgemeine Seiten - mit **Suchfeld**, Kategorien,
  scrollbaren Artikeln und Tastenkappen-Chips, in allen fünf Sprachen.
  Erreichbar über den Sidebar-Button **„Wiki / Hilfe"** und aus dem
  Vorspiel-Screen jedes Spiels (öffnet direkt dessen Seite).
- **Erfolge & Statistiken**: **83 Erfolge** in drei Kategorien - sammlungsweite
  Ziele (Partien, Spielzeit, Rekorde, Siege …), ein **Punkte-Meilenstein je
  Spiel** und **besondere Momente** (Schachmatt gegen die KI, KNIFFEL, die
  2048er-Kachel, ein Wordle in 2 Versuchen …). Beim Freischalten erscheint ein
  **goldener Toast mit Fanfare** - auch mitten im Spiel; alte Highscores werden
  beim ersten Start automatisch angerechnet. Dazu ein **Statistik-Reiter** mit
  Gesamtspielzeit, Partien, Siegen, Rekorden, Lieblingsspiel und einer nach
  Spielzeit sortierten **Pro-Spiel-Tabelle** (Klick auf eine Zeile öffnet das
  Spiel). Erreichbar über den Sidebar-Button **„Erfolge & Statistik"**.

### Bedienung

- Spiel links im Menü per Button wählen. Danach erscheint ein **Vorspiel-Screen**:
  **Einzelspieler** oder **Mehrspieler** wählen, zu den **Optionen** gehen oder
  zurück. Pfeile/Maus zum Wählen, Enter startet.
- **ESC** = Pause / weiter (in Menüs: zurück).
- **F11** (oder Button „Vollbild an/aus") = Vollbild ein/aus. Das Pygame-Display
  bleibt eingebettet und wird seitenverhältnistreu hochskaliert (schwarze Ränder
  bei abweichendem Seitenverhältnis). Das Fenster lässt sich frei skalieren.
- **„Zurück zum Menü"** beendet das Spiel und speichert den Highscore.
- **„Beenden"** schließt Pygame und Tkinter sauber.

### Optionen, Steuerung & Sound

Über den Button **„Optionen / Steuerung"** (links) oder aus dem Vorspiel-Screen
öffnet sich der Options-Bildschirm. Er ist in **drei Reiter** gegliedert
(**Allgemein / Steuerung / Erscheinungsbild**; wechseln per Klick oder
Tab-Taste):

- **Allgemein**: **Sound** an/aus, **Lautstärke** und **Haptik**
  (Gamepad-Vibration, nur mit angeschlossenem Controller wirksam) sowie
  **Auto-Auflösung**, **Auflösung**, **FPS** und **Sprache** – jeweils per
  Links/Rechts umschalten.
- **Steuerung**: **Vorlagen** (*WASD + Pfeile*, *WASD + IJKL*, *Pfeile + WASD*)
  und **jede einzelne Taste** für Spieler 1 und Spieler 2 frei belegen:
  Zeile wählen, Enter drücken, gewünschte Taste drücken (Esc bricht ab).
- **Erscheinungsbild**: das **UI-Design** wählen – **UI v4.1** (Standard: wie
  UI v4, aber lebendiger – dezente Sterne sowie Saturn und ein Schwarzes Loch
  im Hintergrund des Startbildschirms), **UI v4** (komplett ruhiges, flaches
  Graphit-Design mit einem Indigo-Akzent) oder **UI v3** (die bisherige
  klassische UI mit Sternenhimmel, Aurora-Lichtern und Glow-Effekten). Alle
  Karten zeigen eine kleine Vorschau; die Wahl wirkt sofort auf die komplette
  Oberfläche (Spielfläche **und** Sidebar) und wird gespeichert.

Einstellungen werden dauerhaft in `settings.json` gespeichert. Im **Einzelspieler**
steuern beide Belegungen dieselbe Figur (Standard: WASD *und* Pfeile), im
**Mehrspieler** je eine. Alle Spiele haben **Soundeffekte** (prozedural erzeugt,
keine Extra-Dateien nötig), die sich global stummschalten lassen.

### Projektstruktur

```
install-python.bat  Windows-Einrichtung: Python 3.13 + .venv + pygame
start.bat            Startskript (Windows)
start.sh             Startskript (Linux / macOS / Git Bash)
pyinstall.bat        EXE-Build (Windows): packt alles in eine builds\PyGameZ.exe
main.py              Tkinter-Oberfläche, Pygame-Einbettung, zentrale Game-Loop
game_base.py         Game-Basisklasse (update/draw/handle_event) + InputEvent + Helfer
settings.py          Einstellungen (Sound/Haptik/Tastenbelegung) laden/speichern (JSON)
audio.py             Prozedurale Soundeffekte + Gamepad-Rumble
menu.py              Sprach-, Vorspiel- (Modus) und Options-Screen (Sound/Steuerung)
highscore.py         Laden/Speichern der Highscores (Abschnitt in mem.json)
store.py             Zentrale Speicherdatei mem.json (Abschnitte: mem, highscores, stats, achievements)
stats.py             Spielerstatistiken (Partien, Spielzeit, Siege, Rekorde) je Spiel
achievements.py      Erfolge: Definitionen, Freischalt-Logik, Toast-Einblendung
progress.py          Erfolge-&-Statistik-Screen (zwei Reiter, scrollbar)
prestige.py          Prestige-System für Snake
competitive.py       Kennzahlen für den Competitive-Modus von Snake (Level, Slot, Wett-Äpfel)
ngb.py               Visuelle Personalisierung ("Mods"): Kopffarbe + Koordinaten-Raster + Menü (mem-ngb.json)
i18n.py              Übersetzungs-Engine (lädt lang/*.json, t("schlüssel"))
lang/
  de.json  en.json  fr.json  es.json  pt.json   Sprach-Strings (ein Platzhalter-Schlüssel je Text)
lamawiki/
  lamawiki.py          In-Game-Wiki (Suche, Kategorien, Artikel-Renderer)
  de.json  en.json  fr.json  es.json  pt.json   Wiki-Inhalte (eine Seite je Spiel + Allgemeines)
games/
  snake.py  pong.py  airhockey.py  tictactoe.py  breakout.py  tetris.py
  invaders.py  asteroids.py  pacman.py  flappy.py  doodle.py
  game2048.py  minesweeper.py  sudoku.py  sudoku_gen.py
  frogger.py  memory.py  solitaire.py  cards.py  aimtrainer.py
  connect4.py  tanks.py  blackjack.py  tunnelracer.py
  labyrinth.py  maze_gen.py  reversi.py  kniffel.py  wordle.py
  trexrunner.py  dame.py  poker.py  chess.py  muehle.py
  simon.py  billiard.py  slidepuzzle.py  mastermind.py
  bubbleshooter.py  hangman.py  hangman_words.py  blockjump.py
```

Die gewählte Sprache wird in `mem.json` gespeichert (im Abschnitt `mem`, neben
dem Abschnitt `highscores` in derselben Datei) und beim nächsten Start
automatisch geladen.

### Plattformhinweise

Die Anzeige läuft **off-screen**: pygame nutzt den Dummy-Video-Treiber
(`SDL_VIDEODRIVER=dummy`), rendert also in eine Surface, und jedes Frame wird als
Bild in ein Tkinter-Widget gezeichnet. Es gibt **kein natives SDL-Fenster**, das
mit Tkinter um Größe/Position kämpfen könnte. Dadurch verhält sich das Fenster
überall gleich und stabil:

- **Windows**: Der Prozess wird zusätzlich DPI-aware gemacht, damit die Anzeige
  auf skalierten Displays (125/150/200 %) scharf ist und nicht „rüttelt".
- **Linux/X11 & Wayland**: funktioniert ohne Sonderfälle (kein `SDL_WINDOWID`).
- **macOS**: funktioniert ebenfalls (früher wurde das eingebettete Fenster hier
  gar nicht angezeigt).

---

### Installations-Guide

Voraussetzung: **Python 3.9+** (empfohlen 3.12 oder 3.13) und **pygame ≥ 2.6**.

#### Windows (empfohlen: automatisch)

1. Projektordner öffnen und **`install-python.bat`** per Doppelklick starten.
   Das Skript
   - prüft, ob **Python 3.13** vorhanden ist, und installiert es sonst über
     **winget** (`winget install Python.Python.3.13`),
   - erstellt die virtuelle Umgebung **`.venv`**,
   - installiert **pygame** aus `requirements.txt`.
2. Anschließend die Sammlung mit **`start.bat`** starten (Doppelklick).

> Hinweis: Meldet das Skript „in diesem Fenster noch nicht verfügbar", wurde
> Python frisch installiert – einfach **ein neues Terminal/Fenster** öffnen und
> `install-python.bat` noch einmal ausführen. Ist **winget** nicht vorhanden,
> Python 3.13 manuell von <https://www.python.org/downloads/> installieren und
> dabei **„Add python.exe to PATH"** anhaken.

#### Windows / Linux / macOS (manuell)

```bash
# 1. Python prüfen (3.9+)
python --version

# 2. Virtuelle Umgebung anlegen und aktivieren
python -m venv .venv
#   Windows (cmd):        .venv\Scripts\activate
#   Windows (PowerShell): .venv\Scripts\Activate.ps1
#   Linux / macOS:        source .venv/bin/activate

# 3. Abhängigkeiten installieren
pip install -r requirements.txt
#   oder:  pip install "pygame>=2.6" (or pygame-ce)
#                                     pip install pygame-ce

# 4. Starten
python main.py
```

#### Linux / macOS mit start.sh

```bash
# Python + venv wie oben (Schritte 2 und 3) einrichten, dann:
chmod +x start.sh      # einmalig, falls noch nicht ausführbar
./start.sh
```

Unter Linux installiert man Python bei Bedarf über den Paketmanager, z. B.
`sudo apt install python3 python3-venv python3-pip` (Debian/Ubuntu), unter macOS
z. B. `brew install python`.

#### Andere Python-Version verwenden

`install-python.bat` richtet standardmäßig Python 3.13 ein. Wer 3.12 (oder eine
andere Version) bevorzugt, ändert in der Datei die Zeile `set "PYVER=3.13"` auf die
gewünschte Version und die winget-ID entsprechend (`Python.Python.3.12`).

#### Eigenständige EXE bauen (Windows)

```bat
pyinstall.bat         :: baut builds\PyGameZ.exe (alles in einer Datei)
```

`pyinstall.bat` nutzt die `.venv` (und erstellt sie bei Bedarf), installiert
**PyInstaller** automatisch nach und packt das komplette Spiel - Python,
pygame, alle Spiele, Sprachen, Wiki und Logos - in **eine einzige
`PyGameZ.exe`** im Ordner **`builds\`**. Die Datei läuft auf jedem Windows-PC
ohne installiertes Python und lässt sich frei kopieren. Einstellungen und
Highscores (`settings.json`, `mem.json`, `mem-ngb.json`) legt die .exe beim
Spielen neben sich an.

#### Fehlersuche

- **`pygame` nicht gefunden** → venv aktiviert? Schritt 3 wiederholen
  (`pip install -r requirements.txt`).
- **`python` wird nicht erkannt (Windows)** → Python wurde ohne „Add to PATH"
  installiert; neu installieren und Haken setzen, oder `py` statt `python` nutzen.
- **Kein Ton** → in den Optionen „Sound" prüfen; Haptik wirkt nur mit Controller.
- **Fenster/Einbettung unter Linux** → siehe *Plattformhinweise* (Wayland/XWayland).

<div align="right"><b><a href="#pygamez">↑ nach oben / back to top</a></b></div>

---

<a name="-english"></a>

## 🇬🇧 English

A desktop game collection in Python: **Tkinter** provides the window and menu,
**Pygame** is embedded as the game display inside the Tkinter window. Forty-two
games with shared options, freely rebindable controls, high scores, procedural
sound effects and, for some titles, a multiplayer mode. The interface is
**multilingual** – **14 languages** (German / English / French / Spanish /
Portuguese / Polish / Turkish / Danish / Norwegian / Swedish / Finnish / Czech /
Slovenian / Croatian); the language is chosen on a **welcome screen** at first
launch, which also lets you set the **resolution** and **sound** (off by
default); apart from the three main languages, all the others (Spanish,
Portuguese and the nine additional ones) sit behind the **"More"** button.
Everything can be changed later in the options.

### Quick start

#### Windows

```bat
install-python.bat    :: one-time: set up Python 3.13 + .venv + pygame
start.bat             :: launch the game collection
```

#### Linux / macOS / Git Bash

```bash
./start.sh            # starts with .venv, otherwise system python3
```

`start.bat` / `start.sh` automatically use the virtual environment `.venv` if
present, otherwise the system Python. A detailed step-by-step guide is at the very
bottom under **[Installation Guide](#installation-guide)**.

### The games

| Game         | Modes           | Short description |
|--------------|-----------------|-------------------|
| **Snake**    | 1 / 2 players   | Deluxe Snake with 2D & 3D view, boost, 6 game modes (incl. Competitive), golden apples and prestige |
| **Pong**     | 1 / 2 players   | Classic vs. AI or player 2, switchable movement mode |
| **Air Hockey** | 1 / 2 players | 2D physics with momentum transfer, mouse control, AI and power-ups |
| **Tic-Tac-Toe** | 1 / 2 players | m,n,k game on 3x3 to 9x9, three AI strengths **or** local X vs. O |
| **Breakout** | 1 player        | Brick breaker with brick types, power-ups, combos and many levels |
| **Tetris**   | 1 / 2 players   | Classic or Versus (two fields side by side) |
| **Invaders** | 1 player        | Space Invaders: clear the waves, protect your lives |
| **Asteroids** | 1 / 2 players  | Inertia physics, waves, UFOs, power-ups, hyperspace - solo or co-op duel |
| **Pac-Man**  | 1 player        | Faithful clone: 4 ghost AIs, power pills, tunnel, fruit, levels |
| **Flappy Bird** | 1 player     | Gravity flight through pipes, coins, shield, day/night, medals |
| **Doodle Jump** | 1 player     | Auto-jump upward, platform types, springs, propeller, monsters |
| **2048**     | 1 player        | Number-sliding puzzle, goal: the 2048 tile |
| **Minesweeper** | 1 player     | The classic with safe first click, chording, smiley and best times |
| **Sudoku**      | 1 player     | 400 seeded levels (4 difficulties x 100), 4 assist modes with score multiplier, notes, hints, 3-error limit |
| **Frogger**     | 1 player     | Road + river + 5 bays, bonus fly, crocodiles, time limit, 3 difficulties |
| **Memory**      | 1 / 2 players | Find pairs on 4x4 up to 8x6, flip animation, solo scoring or duel |
| **Solitaire**   | 1 player     | 5 variants (Klondike, Spider, FreeCell, Pyramid, TriPeaks) with drag & drop and undo |
| **Aim Trainer** | 1 player     | Chill 3D target shooting: mouse steers the camera, 4 modes (precision/reflex/moving/chill), 3 themes incl. a black hole |
| **Connect Four** | 1 / 2 players | The classic with falling-disc animation: 3 AI strengths (minimax) or a local duel |
| **Tank Duel**    | 1 / 2 players | 2D arena duel with ricochet shots, power-ups, 4 arenas, AI with 3 strengths |
| **Blackjack**    | 1 player     | Casino blackjack with a 4-deck shoe, double/split, 3:2 blackjack and a persistent chip balance |
| **Tunnel Racer** | 1 player     | 3D neon tube flight: endless mode + 30 levels, key or mouse steering, motion blur |
| **3D Maze**      | 1 player     | First-person raycaster (Wolfenstein style) with 50 seeded levels, orbs, minimap - or a 2D top-down view |
| **Reversi**      | 1 / 2 players | Othello on 8x8: trap and flip discs, 3 AI strengths (minimax) or a local duel |
| **Kniffel (Yahtzee)** | 1 / 2 players | Dice classic with 13 categories, upper bonus and Yahtzee; high-score chase or 2-player hotseat |
| **Wordle**       | 1 player     | Guess the 5-letter word in 6 tries, endless streak, colour hints, 5 languages |
| **T-Rex Runner** | 1 player     | Endless desert run: variable jump, duck, cacti & pterodactyls, day/night cycle, rising speed, 3 difficulties |
| **Draughts (Dame)** | 1 / 2 players | 3 rule sets (German 8×8, International 10×10, Checkers), forced captures & flying kings, 3 AI strengths (minimax) or a local duel |
| **Poker**        | 1 player     | 3 selectable variants: Texas Hold'em vs AI, 5 Card Draw and Video Poker; betting rounds, blinds, persistent chip bankroll |
| **Chess**        | 1 / 2 players | Full rules (castling, en passant, promotion, mate/stalemate/draws), 6 AI strengths (minimax + alpha-beta) or a local duel, colour choice |
| **Nine Men's Morris** | 1 / 2 players | Placing/moving/flying phases, mills & captures, optional flying rule, 3 AI strengths or a local duel |
| **Simon**        | 1 / 2 players | Senso memory game: Classic/Speed/Reverse/Mixed modes + Duel, sound off/on/mixed, 4/6/9 pads, best per mode |
| **Billiards**    | 1 / 2 players | 8-ball, 9-ball & practice in 2D, fixed 3D view or free-orbit 3D camera; smooth physics, aim assist, 3 AI strengths |
| **Sliding Puzzle** | 1 player    | 15-puzzle in 3x3/4x4/5x5: slide the numbered tiles into the gap, click or arrow control, score from moves & time |
| **Mastermind**     | 1 player    | Crack the secret colour code (3 modes: 4×6, classic, 5×8), black/white feedback pegs, endless streak high score |
| **Bubble Shooter** | 1 player    | Puzzle Bobble clone: shoot matching colours into groups of three, wall bounces, falling clusters, 3 difficulties |
| **Hangman**        | 1 player    | Guess the word before the gallows is finished; on-screen keyboard, per-language word lists, 3 length modes, endless streak |
| **Block Jump**  | 1 player        | 3D Minecraft-style platformer: textured voxel world, Steve figure, ladders, fences & slime blocks, first/third-person camera, seed-generated parkour levels |
| **Tower Defense** | 1 player      | Fend off endless waves on 4 maps: up to 11 tower types with upgrades, selling & A/B specialisation, bosses, 3 modes, active abilities |
| **Minigolf**    | 1 / 2 players   | 360 holes across 40 courses (18 hand-built, 342 seed-generated): sand, ramps, water, rubber bumpers, windmills & moving blocks; scorecard with par and hole-in-one bonus |
| **Pinball**     | 1 / 2 players   | Pinball machine with 3 tables: pop bumpers, slingshots, drop targets, L-A-M-A lanes, multiball with jackpot, ball save, nudge & tilt |
| **Bowling**     | 1 / 2 players   | 10 frames with official strike/spare scoring, real pin physics, hook spin and a perspective lane view, 3 difficulties |

**Multiplayer (2 players local)** is available for **Snake**, **Pong**,
**Air Hockey**, **Tic-Tac-Toe**, **Tetris (Versus)**, **Asteroids (co-op
duel)**, **Memory (duel)**, **Connect Four**, **Tank Duel**, **Reversi**,
**Kniffel**, **Draughts**, **Chess**, **Nine Men's Morris**, **Simon (duel)**,
**Billiards**, **Minigolf**, **Pinball** and **Bowling**. The mode is
chosen right in the pre-game screen (*Single-player / Multiplayer*).

#### Feature details per game

**Snake**
- **NEW - 3D view** (key **V** in setup or click *View*): the board is rendered
  as a real-time 3D scene - a **chase camera** floats behind the snake and
  steering is **relative to the view** (left/right = turn, two quick presses =
  U-turn). With distance fog, a starry sky, checkerboard floor, border walls,
  rotating food crystals, 3D particles and camera shake on crash; after game
  over the camera slowly orbits the snake. Boosting widens the field of view.
  Available in 3D: *Classic* and *Obstacles* (walls are always solid there,
  3D is single-player only). The view is remembered in `settings.json`.
- **NEW - 3D camera options** (in the 3D setup click the *3D camera / smooth shake*
  row, or key **K**): a dedicated menu with **smooth shake** (a gentler camera, far
  less jitter when moving/turning), adjustable **field of view (FOV)** and **camera
  height**, plus a **shake when turning** toggle (screen shake on left/right turns
  on/off). All remembered in `settings.json`.
- **Boost**: **hold** the boost key = turbo (double speed), consumes stamina
  (bar); once empty, the boost switches off and recharges. Default P1 =
  Space/Left-Shift, P2 = Enter/Right-Shift.
- **6 game modes** (selectable in setup): *Classic*, *Speed Rush* (gets faster
  with every apple), *Obstacles* (deadly blocks), *Portals* (teleporter pairs),
  *Time Attack* (60 seconds, as many apples as possible) and *Competitive* (see below).
- **NEW - Competitive** (single-player): endless mode with a **level climb** - you
  start with exactly **one** apple and can't get more at first; the more apples you
  collect overall, the higher your **level**, which keeps adding another simultaneous
  apple to the field and raises the score multiplier.
  **Blue apples** open a **slot machine**: your length is the stake, the reel result
  multiplies or shrinks it and briefly makes **extra apples** spawn (jackpot on three
  matching symbols). **Purple apples** (gambling) put a share of your **size** on the
  line and multiply that part randomly, the rest stays safe (new size = size·(1-p) +
  size·p·factor): **normal** stakes a fixed 50 % with **x0.5 .. x1.5**, **HARDCORE** is
  riskier with a **75-90 %** stake and **x0.25 .. x2.25**. Your **size** is shown as a
  **decimal in the top-left** and carried over exactly, so further bets build on it.
  There are **15 levels** (multiplier up to x16, up to 16 apples at once); the levels live in
  `games/levels/snake-comp.json` and can be extended there without touching code, the rest
  of the tuning lives in `competitive.py`.
- **NEW - HARDCORE** (toggle in the Competitive setup, key **H**): every **boost eats
  your snake's length**; a red glowing **HARDCORE label** marks the mode. Competitive
  only; length never drops below the minimum. Remembered in `settings.json`.
- **Golden apples** (temporary) give lots of points and instantly refill the boost.
- Optional **wrap-around walls**, bonus apples, **prestige** (single-player, key **P**).
- **NEW - Personalize** (brush button in the very top-right of setup, or key **C**):
  a visual-only menu ("mods" that *never* change gameplay) with two tabs:
  - **Head**: the snake's **head color** - 4 blue-teal presets (from more blue to more
    teal), red, orange and a **custom color** via RGB sliders.
  - **Grid (signpost)**: overlays a **coordinate grid** on the field - **row numbers**
    (on the left and right edges) and **column letters** (top/bottom). So on large
    boards you instantly see that e.g. the apple at *8a* is in the
    same row *8* as your own position *8z*. The color sequence (5 presets + two custom
    colors A/B) sets the color theme.
  - **Banner**: turn the multiplier banner (e.g. from the purple apple) **on/off** and
    adjust its **size** (smaller/larger) and **opacity** (more transparent) - with a
    live preview.
  Everything is stored in `mem-ngb.json`; all visual personalization runs through the
  `ngb.py` module.
- Look: rounded snake with eyes (head is teal by default), boost glow, particles.

**Pong**
- Single-player vs. AI, multiplayer = player 2 on the right. First to 5 points.
- **Movement mode switchable per control set**: *Continuous* (press once -> keeps
  moving, default) or *Hold* (moves only while held).
  Toggle: **X** = control set 1, **N** = control set 2 (remembered in `settings.json`).
- Ball physics with acceleration and angle depending on the hit point.

**Air Hockey**
- **Real 2D physics**: round mallets and puck with momentum transfer - the puck
  picks up the mallet's velocity on impact; walls with restitution, light ice
  friction, goals as openings in the side walls.
- **Mouse control** in single-player: the mallet follows the mouse (any key
  switches back to keyboard). Keyboard: direction keys in 8 directions,
  multiplayer = P1 left (WASD), P2 right (IJKL).
- **AI with three strengths** (Easy/Medium/Hard): defends its goal, attacks in
  its own half and curves around the puck to avoid own goals.
- **Power-ups** (can be disabled): *XL* (bigger mallet), *TOR* (opponent's goal
  shrinks), *>>* (faster mallet) - they belong to the player who touched the
  puck last.
- Setup: difficulty, **goals to win** (3/5/7/10), power-ups on/off (saved in
  `settings.json`). After each goal the conceding player serves.
- Look: puck light trail, particles, pulsing goal mouths, effect badges.

**Tic-Tac-Toe**
- Setup: difficulty (Easy/Medium/Hard) and board size 3x3..9x9; win length
  K = 3 (3x3), 4 (4x4), otherwise 5.
- **1 player** vs. AI (Hard on 3x3 is unbeatable) **or 2 players** local
  (X vs. O, taking turns by clicking). On game over: Enter/click = new round,
  **S** = settings.

**Breakout**
- Brick types: Normal, **Steel** (indestructible), **Bomb** (explodes), **Gold**
  (extra points).
- Power-ups: laser, fireball, sticky, shield, coin and more; **combo multiplier**.
- Effects: particles, ball trails, screen shake, score pop-ups, many level patterns.
- Setup: **1/2/3** = difficulty, **Left/Right** = ball color, **Up/Down** =
  starting level, **M** = layout. Game: mouse/arrows, **Space** launches the ball
  (fires laser), **P/Esc** = pause.

**Tetris**
- Left/Right to move, Up = rotate, Down = soft drop, Action = hard drop.
- Full rows give points, every 10 rows the level increases.
- **Versus**: whoever's stack hits the top first loses.

**Invaders** – two modes (selectable in the pre-game screen):
- **Classic**: classic alien block; then selectable in the setup screen:
  **Movement** (left/right only *or* free with WASD) and **Aiming** (always
  upward *or* toward the **mouse** – then you shoot wherever the cursor is).
  Destroyed aliens sometimes drop power-ups.
- **Arena (free)**: free movement in all directions, enemies pour in from every
  edge; you aim in the direction of movement, switch weapon with **1–4**.
Shared: level system with a **boss** every 4th level, four weapons (blaster,
spread shot, rapid fire, laser), power-ups (extra life, shield, weapon upgrade),
explosion effects, high score.

**Asteroids**
- **Inertia physics**: up = thrust in the facing direction, left/right = rotate,
  the ship keeps drifting (slight damping); everything wraps around the screen
  edges. Classic **vector look** with thruster flame and a starry sky; every
  rock has its own random polygon shape.
- Rocks shatter into two smaller ones (3 sizes, **20/50/100 points**),
  **waves** with growing counts and a banner announcement.
- **UFO** (can be disabled): crosses the screen periodically and aims at the
  ships (aim error depends on difficulty) - 200 points for shooting it down.
- **Power-ups** (can be disabled), dropped by destroyed rocks: **S**hield (6 s
  invulnerable), **T** = triple shot, **R** = rapid fire.
- **Hyperspace** (down key): emergency jump to a random position with a 4 s
  cooldown - and a 12 % risk of blowing up on arrival.
- 3 lives, safe respawning with invulnerability blinking, **extra life every
  5000 points**; explosion particles and camera shake.
- **Co-op duel** (multiplayer): both ships fly at the same time with separate
  lives and scores - whoever scores more wins.
- Setup: difficulty, UFOs on/off, power-ups on/off (saved in `settings.json`).

**Pac-Man**
- **Classic 28x31 maze** in a neon look with pellets, 4 power pills, side
  tunnel warps and a ghost house in the middle.
- **Four ghosts with the original behaviours** (target-tile AI): *Blinky*
  chases directly, *Pinky* ambushes (4 tiles ahead), *Inky* uses a vector
  through Blinky, *Clyde* backs off when close.
- **Scatter/chase phases** alternate (ghosts reverse on each switch); a
  **power pill** turns ghosts blue and edible (chain 200/400/800/1600), then
  their eyes return to the house.
- Ghost house with **staggered release**, **fruit** bonuses (per level),
  **3 lives**, **extra life at 10,000**, level system (gets faster), death
  animation, READY/GAME OVER screens.
- Setup: **difficulty** (Normal/Hard/Extreme) – ghost speed & frightened time.
- Controls: **arrows or WASD**.  Enter = new, S = setup.

**Flappy Bird**
- **Gravity physics**: Space / Up / W / **mouse click** makes the bird flap;
  it tilts based on climb/fall speed.
- Endless **pipe pairs** with a gap (+1 per pipe); **coins** (bonus) and a
  **shield** power-up (survive one hit) appear in the gaps.
- **Day/night themes** change with the score; drifting clouds (parallax),
  scrolling ground.
- Difficulty (Easy/Normal/Hard): gap size, speed, pipe spacing – the gap
  narrows slightly as the score climbs.
- **Medals** (bronze/silver/gold/platinum) on game over, crash animation with
  camera shake, high score.

**Doodle Jump**
- The doodler **auto-jumps** on landing; you only steer left/right (with
  inertia), the edges wrap around, and the camera scrolls up as you climb.
- **Platform types**: green (normal), blue (moving), brown (breaks), white
  (vanishes). **Springs** give a super bounce, the **propeller hat** carries
  you up briefly (and makes you invincible).
- **Monsters**: contact is deadly – but you can **shoot** them with Up / Space
  (bonus points).
- Score = height reached; difficulty rises with height. High score.
- Controls: left/right = move, Up / Space = shoot.

**2048** – arrows/WASD slide all tiles; equal numbers merge.

**Minesweeper**
- Three levels: **Beginner** (9x9, 10 mines), **Intermediate** (16x16, 40),
  **Expert** (30x16, 99) - the **best time per level** is saved and shown in
  the setup.
- The **first click is always safe** (mines are placed afterwards, the 3x3
  area around the click stays clear).
- **Left click** = reveal, **right click** = flag (optionally with a question
  mark cycle), **F** = flag under the cursor, **R** = new game.
- **Chording**: clicking a satisfied number reveals the remaining neighbors.
- Classic HUD: mine counter, **clickable smiley** (surprised/sunglasses/dead),
  timer; wrong flags are crossed out at the end, confetti on victory.
- Points = level base value minus seconds.

**Sudoku**
- **400 levels**: 4 difficulties (Easy/Normal/Hard/Expert) x 100 levels. The
  puzzles are **seed-generated with a unique solution** - level 12 of "Hard"
  is the same puzzle on every PC. Solved levels are saved and ticked off in
  the level select.
- **4 game modes** (chosen before starting) with a score multiplier:
  **Classic** (x2.0 - no assists), **Notes** (x1.5 - + pencil notes),
  **Comfort** (x1.0 - + wrong digits red, conflict and same-digit
  highlighting, correct entries lock in), **Assist** (x0.7 - + hint key,
  max. 3).
- Every entry is checked against the solution immediately; with the
  **3-error limit** enabled (setup option) the third mistake ends the game.
- Controls: arrows/WASD = cell, **1-9** = digit (numpad too),
  **0/Backspace/right click** = erase, **N** = notes, **H** = hint,
  **R** = restart level, **Q** = level select; fully playable with the mouse
  (number pad on the right). After the game ends, **A** hides the banner
  and reveals the full **solution** on the board (A again = back).
- Points = (difficulty base - time - errors - hints) x mode multiplier.

**Frogger**
- 5 traffic lanes (cars/trucks) and 5 river lanes (logs, turtles that **dive**
  on higher levels); 5 home bays at the top - fill all = next level, everything
  speeds up.
- Extras: **bonus fly** (+200) in empty bays, **crocodiles** occupy bays on
  higher levels, **time-limit bar** per frog, extra life at 10,000.
- 3 difficulties (speed, traffic density, time); points per new row,
  bay = 50 + time bonus, level complete = +1000.

**Memory**
- Board sizes **4x4, 6x6, 8x6**; motifs are shape-color combinations drawn
  entirely with primitives; **flip animation**, mismatched pairs flip back
  automatically.
- **Solo**: base - 15 per move - 2 per second (min. 100). **Duel** (local):
  alternating turns, a match grants another turn, most pairs wins.

**Solitaire**
- **5 variants** on the pre-game screen: Klondike (draw 1/3 option), Spider
  (1/2/4 suits), FreeCell (supermove limit), Pyramid (13-pairs, 2 redeals)
  and TriPeaks (±1 chain with combo multiplier).
- **Drag & drop** or click-click, **right click** = to foundation,
  **U** = unlimited undo, **R** = new deal, Space = stock.
- Cards are rendered without image files (`games/cards.py`); all variants
  share one high-score list with variant-specific formulas.

**Aim Trainer**
- **Real software 3D** (like Snake's 3D mode): fixed crosshair at the screen
  center, **direct 1:1 mouse look like a shooter** (pointer capture: the
  cursor is grabbed inside the window, Esc releases it; adjustable
  sensitivity, unlimited yaw, pitch ±60°). Left click shoots exactly through
  the center, with muzzle flash, tracer and hit particles.
- **4 modes**: precision (60 s, 3 orbs, accuracy bonus), reflex (30 single
  targets, reaction-time stats), moving targets (paths + combo multiplier up
  to x4) and chill (endless, no penalty, **E** ends the session).
- **3 themes** (in the setup, saved): **space** with a star sphere, a
  **black hole with a glowing ring** and a planet (default), a neon arena
  with floor grid and synthwave sun, and an indoor shooting range.
- Sensitivity can be changed mid-game with **+/-**; plus an **adjustable
  motion blur** (0-80%) for extra chill visuals - both are saved.

**Connect Four**
- 7x6 board with a **falling-disc animation**, hover preview and a pulsing
  winning line; mouse, arrow keys or direct pick **1-7**.
- **3 AI strengths** (minimax with alpha-beta search): Easy deliberately
  misses threats, Medium blocks reliably, Hard plans deep ahead - or
  **2 players** locally on the same device.
- The starting player alternates every round; the high score counts your
  **wins against the AI** in one session.

**Tank Duel**
- 2D arena duel: **shots bounce off walls once** (ricochet) - hit around
  corners (or yourself!). First to 5 rounds with a countdown.
- **4 arenas** (Open, Cross, Pillars, Maze) or random rotation;
  **power-ups**: rapid fire, shield, triple shot.
- **AI with 3 strengths** - the hard one leads its shots and banks them off
  walls on purpose - or **2 players** on one keyboard (P1 WASD+Space,
  P2 arrows+Enter).

**Blackjack**
- Real casino rules: **4-deck shoe**, dealer stands on 17, **blackjack pays
  3:2**, dealer peek on ace/10; **double down** and **one split** (split aces
  get one card each).
- **Persistent chip balance**: start with 500, balance and **record** survive
  every restart (`mem.json`); below 10 chips you get 500 fresh ones - the
  record stays.
- Played via chip buttons and keys (**H**it/**S**tand/**D**ouble/split
  **X**, **1-4** = bet, Enter = deal) with card animations and a hole-card
  flip.

**Tunnel Racer**
- **3D neon tube flight** (software renderer like the Aim Trainer): bars,
  blocks and **ring gates to thread through**, coins on the racing line.
- **Two modes**: endless (speed rises to a cap, high score) and **30 seeded
  levels** with a finish line, time bonus and ticked-off progress.
- **Key steering** (default) or **direct mouse steering** (pointer capture,
  key **C**); plus adjustable **motion blur** (key **B**, 0-80%) - everything
  is saved.

**3D Maze**
- **First-person raycaster in Wolfenstein style** (DDA, distance fog,
  sprites) with mouselook + WASD, a **minimap** (key **M**) and a green
  pulsing exit - or a classic **2D top-down view** (key **V** in the setup).
- **50 seeded levels** that keep growing; the exit always sits at the point
  farthest from the start, **orbs** along the way give bonus points.
- Scoring: 500 per level + 100 per orb + time bonus; solved levels are
  ticked off and the session total becomes the high score.

**Reversi**
- **Othello on 8x8**: place discs that trap the opponent's rows and flip
  everything enclosed; illegal moves are blocked and a turn with no legal move
  is **passed automatically**.
- **Single player vs. the AI** (3 strengths: negamax with alpha-beta,
  positional weighting + mobility) **or a local duel**, Black vs. White.
- Legal squares are highlighted; play with the **mouse** or the selection frame
  (arrows + Space/Enter). Every win against the AI counts one point toward the
  high score.

**Kniffel (Yahtzee)**
- **Dice classic**: 5 dice, up to 3 rolls per turn, **hold** dice individually,
  then book one of the **13 categories** (with a live preview of the possible
  score).
- Full scoresheet: upper section with **63-point bonus (+35)**, three/four of a
  kind, full house, small/large straight, **Yahtzee (50)** and Chance.
- **Single player as a high-score chase** for the highest total, or **2-player
  hotseat** with two sheets side by side; play by mouse or keys (Space, 1-5,
  arrows, Enter).

**Wordle**
- Guess the **5-letter word in 6 tries**; colour feedback (green/yellow/grey)
  with correct **duplicate-letter counting** and an on-screen keyboard that
  colours in.
- **Endless streak**: each solved word scores points (fewer guesses = more), the
  first unsolved word ends the run - total = high score.
- **Per-language word lists** (A-Z only); guesses are not checked against a
  dictionary. Type on the keyboard or click the on-screen keys.

**Chess**
- **Full chess**: every piece move including **castling**, **en passant** and
  **pawn promotion** (choose the piece); **check, checkmate and stalemate** plus
  draws by the **fifty-move rule**, **threefold repetition** and **insufficient
  material**.
- **6 AI strengths** from *Beginner* to *Master* (negamax with alpha-beta, move
  ordering, piece-square tables, quiescence search); a **time budget** keeps
  every AI move smooth. **Colour choice** white/black or a **local duel**.
- Legal moves are highlighted, check is flagged; play with the **mouse** or a
  selection cursor. Each win against the AI scores one high-score point.

**Nine Men's Morris**
- **Mills** with all three phases: **placing** (9 pieces each), **moving** along
  the lines and **flying** once down to 3 pieces (can be switched off).
- A completed **mill** removes an opponent's piece (preferably one outside a
  mill); you lose when reduced below 3 pieces or unable to move.
- **3 AI strengths** (minimax with alpha-beta, phase-aware evaluation) or a
  **local duel**; with move hints, mill highlighting and a piece counter.

**Simon**
- **Senso memory game**: the lit sequence grows every round and must be repeated
  exactly.
- **Modes**: *Classic*, *Speed* (gets faster), *Reverse* (backwards), *Mixed*
  (mode rotates each round) and a two-player **Duel** (take turns to add &
  repeat).
- **Sound** *off / on / mixed* (mixed trains your visual AND aural memory),
  **4/6/9 pads** as difficulty; the **best score per mode** is saved. Play with
  the mouse or number keys 1-9.

**Billiards**
- **8-ball**, **9-ball** and a rules-free **practice** mode, versus the AI (with
  aim assist) or **two players locally**.
- **Three freely selectable views**: classic **2D top-down**, a fixed **3D
  angled perspective** with shaded balls and a **free-orbit 3D camera** (right
  mouse button). All motion is time-step based and **smoothly damped** (friction,
  sub-steps to avoid tunnelling).
- **Shooting**: hold the left mouse button to charge power, release to strike;
  an aim line and power meter help. **Ball in hand** after a foul. The view (V)
  is remembered in `settings.json`; frames won count towards the high score.

**Sliding Puzzle**
- 15-puzzle in three sizes: **3×3** (easy), **4×4** (classic) and **5×5** (hard);
  slide the numbered tiles into the free gap.
- Always solvable (shuffled with many random moves). Control by **clicking** a
  tile in the gap's row/column (the whole line slides) or the **arrow keys**.
- Score = a base value per size minus moves and time; solving starts a fresh
  board right away.

**Mastermind**
- Crack the hidden **colour code**; after each guess you get **black** pegs
  (right colour + position) and **white** pegs (right colour, wrong position).
- **3 modes**: Easy (4 pegs / 6 colours / 12 rows), Classic (4/6/10) and Hard
  (5 pegs / 8 colours); duplicate colours allowed.
- Play via the colour palette (click or keys **1–8**), OK/Enter checks the row.
  **Endless streak** like Wordle: every cracked code scores points.

**Bubble Shooter**
- **Puzzle Bobble** on a honeycomb grid: aim with the mouse, shoot bubbles up,
  **three or more of a colour** pop the group.
- Bubbles that lose their link to the ceiling **fall** (bonus); shots **bounce
  off the walls**, with a next-bubble preview.
- **3 modes** (4/5/6 colours, some with descending rows); game over at the red
  line.

**Hangman**
- Guess the word **letter by letter**; each mistake draws a part of the hangman,
  lost after **6 mistakes**.
- **Per-language word lists** (A–Z only), **3 length modes** (short / mixed /
  long); type or click the on-screen keyboard.
- **Endless streak**: every guessed word scores points (more remaining lives +
  longer word = more).

**Block Jump**
- **3D Minecraft-style platformer** (software 3D like Snake's 3D mode): jump
  across a floating **voxel world** of blocks to the glowing goal.
- **Minecraft skin**: every block carries a real **pixel texture** (grass with
  its green overhang, dirt, stone, oak planks, diamond, slime, wood) that is
  split into perspective-correct texel faces – the level of detail follows the
  distance (**T** cycles high / low / off).
- In the same style: a **Steve figure** with face, arms, legs and walk cycle
  (chase camera), the **first-person hand**, a **beacon beam** at the goal,
  spinning **gold ingots** as coins, a square **sun**, drifting **pixel clouds**
  and a HUD with **hearts**.
- Block types: solid blocks (grass/dirt/stone/wood), **ladders** (climb),
  **fences** (jump over), **slime blocks** (launch) and **coins**.
- Camera **first-person like Minecraft by default**, **V** switches to a chase
  camera; **mouse look** with pointer capture, adjustable **motion blur** (**B**)
  and sensitivity (**+/-**).
- **Seed-generated parkour levels** get harder; goal = points + time bonus,
  coins +50, a fall costs a life (start with 3). Controls: WASD/arrows, **Space**
  to jump.

**Tower Defense**
- **Endless wave defence** on **4 maps** (Meadow, Canyon, Crossroads,
  Gauntlet), each with its own path; locked maps unlock via your best wave,
  a **boss** marches in every **8 waves**.
- **3 modes**: Classic (7 towers, the main mode), Compact (4 towers, 2 levels)
  and Maximal (**11 towers**, **A/B specialisation** at top level, special
  enemies, active abilities **Meteor/Frost Nova/Gold Rush**).
- **11 tower types** from arrow to laser & gold bank, up to **3 upgrade
  levels** each, selling refunds 70%; enemies with armor, regeneration,
  splitting, cloaking, heal auras and their own air route.
- **Economy**: gold per kill, wave bonus + 5% interest; points per kill and
  wave. **F** = 2x speed, **G** = ranges, right-click cancels.

**Minigolf**
- **360 holes across 40 courses**: *Classic* and *Pro* with 9 hand-built holes
  each, the **Tour** with 38 courses of 9 seed-generated holes (342 in total) at
  rising difficulty, plus *Random* drawn from everything. Course 7, hole 3 looks
  the same everywhere - nothing has to be stored for it.
- **Surfaces & obstacles**: sand slows you down, ramps accelerate, water costs a
  penalty stroke, rubber bumpers give speed back, and windmills and moving
  blocks are all about timing. The physics runs in sub-steps with friction just
  like Billiards - nothing stutters, nothing tunnels through a rail.
- **Controls**: the mouse aims, holding the left button loads the power and
  releasing putts (arrows + space work too). **R** cancels a charged shot
  without putting. **G** toggles the aim line, **P** the pick-up.
- **Scorecard** on the right with par and strokes per hole; in two-player mode
  each player takes the same hole in turn. Points: 600 per hole, ±300 per stroke
  under/over par, **500 extra for a hole in one**. The lowest stroke count per
  course lives in the `minigolf` section of `mem.json`.
- **Pick-up can be switched off**: by default a hole ends after eight strokes
  and is scored at the minimum. If you would rather keep putting until the ball
  drops, set *Pick up* to OFF in the setup screen (or press **P**).

**Pinball**
- **Three tables**: *Classic* (three pop bumpers, one target bank), *Space*
  (four bumpers in a diamond, two banks) and *Lama* (open playfield, six targets
  in an arc); 3 or 5 balls per game, alternating ball by ball in two-player mode.
- **Everything a machine needs**: a shooter lane with a power meter (pulled too
  softly? the ball rolls back and you may shoot again), two flippers,
  slingshots, drop target banks, four **L-A-M-A** rollover lanes, a saucer with
  ball lock, **multiball with jackpot**, a six second **ball save**, nudging via
  the up key and **TILT** after three hasty nudges.
- **Multiplier up to x5** from cleared target banks and completed lanes; bumpers
  score 100, slingshots 50, targets 250 - during multiball the bumpers pay a
  2,500 point jackpot.
- Flippers run on the bound left/right keys (plus left/right Shift) or the mouse
  (left/right half of the screen). The best result per table is stored in the
  `pinball` section of `mem.json`.

**Bowling**
- **Ten frames by the official rules** including strikes, spares and the bonus
  balls of the tenth frame (maximum: a 300 game). The **scorecard** below the
  header shows every frame with X, / and the running total.
- **Four-step delivery**: position, aiming angle, spin and power. Every slider
  swings on its own and is locked in with the action key - or set by hand with
  left/right, which stops the swinging.
- **Real pin physics**: ten pins as circles with mass that knock each other
  over - a strike is the result of physics, not of luck. The lane is oiled up
  front, so the **hook** only bites in the last third and pulls the ball into
  the pocket.
- Perspective lane view with gutters, aiming arrows and the pin deck; three
  difficulties (*Easy/Normal/Pro*) change slider speed and scatter. The best
  result per difficulty lives in the `bowling` section of `mem.json`.

High scores are stored in the `highscores` section of `mem.json` (next to the
code) – together with the language (section `mem`).

### The interface

The whole interface is drawn from scratch (pure Tkinter + Pygame, no extra
packages) and styled like a modern game launcher:

- **Sidebar game list**: every row has its own **mini pictogram** in the game's
  accent colour, shows the current **high score (★)** and reacts with smoothly
  animated hover effects. The running game stays highlighted; on small windows
  the list **scrolls** with the mouse wheel.
- **Status card** at the bottom left with a **state LED** (grey = menu,
  green = running, gold = paused, red = game over) and a **live FPS readout**.
- **Idle screen** with aurora lights, a parallax star field including shooting
  stars, a floating logo with orbiting sparks, a **clickable game grid** right
  below the logo (all games with a hover effect in their accent colour) and a
  **high-score ticker**.
- **Effects everywhere**: soft screen transitions, sparks when confirming a menu
  entry, **confetti rain on a new high score** and a real **blur** behind the
  pause overlay.
- Each game's **pre-game screen** appears in that game's accent colour and shows
  the previous record as a chip.
- **Unified in-game look**: all 42 games share the menu's theme palette and
  font - HUDs, setup screens and overlays follow the design chosen in the
  options (v4.1 / v4 / Classic), while each playfield keeps its identity
  colours. Every game now handles mid-game resolution changes cleanly, and
  menu names are language-aware (e.g. "Schach" → "Chess" / « Échecs »).
- **In-game wiki** ("LamaWiki"): detailed help for every game (controls, modes,
  scoring, tips) plus general pages - with a **search box**, categories,
  scrollable articles and keycap chips, in all five languages. Reachable via
  the **"Wiki / Help"** sidebar button and from every game's pre-game screen
  (opens that game's page directly).
- **Achievements & statistics**: **83 achievements** in three categories -
  collection-wide goals (games played, play time, records, wins …), one
  **score milestone per game** and **special moments** (checkmating the AI, a
  YAHTZEE, the 2048 tile, a Wordle in 2 tries …). Unlocking shows a **golden
  toast with a fanfare** - even mid-game; old high scores are credited
  automatically on first launch. Plus a **statistics tab** with total play
  time, games, wins, records, favourite game and a **per-game table** sorted
  by play time (clicking a row opens that game). Reachable via the
  **"Achievements & Stats"** sidebar button.

### Controls

- Pick a game via the button in the menu on the left. A **pre-game screen** then
  appears: choose **Single-player** or **Multiplayer**, go to the **options** or
  back. Arrows/mouse to select, Enter to start.
- **ESC** = pause / resume (in menus: back).
- **F11** (or the "Fullscreen on/off" button) = toggle fullscreen. The Pygame
  display stays embedded and is scaled up keeping its aspect ratio (black bars
  when the aspect ratio differs). The window can be resized freely.
- **"Back to menu"** ends the game and saves the high score.
- **"Quit"** closes Pygame and Tkinter cleanly.

### Options, controls & sound

The options screen opens via the **"Options / Controls"** button (on the left) or
from the pre-game screen. It is organized into **three tabs**
(**General / Controls / Appearance**; switch by clicking or with the Tab key):

- **General**: **sound** on/off, **volume** and **haptics** (gamepad vibration,
  only effective with a connected controller) plus **auto resolution**,
  **resolution**, **FPS** and **language** – each toggled with Left/Right.
- **Controls**: **presets** (*WASD + Arrows*, *WASD + IJKL*, *Arrows + WASD*)
  and **rebind every single key** for player 1 and player 2: select a row,
  press Enter, press the desired key (Esc cancels).
- **Appearance**: pick the **UI design** – **UI v4.1** (default: like UI v4
  but livelier – subtle stars plus Saturn and a black hole in the start
  screen's background), **UI v4** (a completely calm, flat graphite look with
  a single indigo accent) or **UI v3** (the previous classic UI with
  starfield, aurora lights and glow effects). All cards show a small preview;
  the choice applies instantly to the whole interface (game area **and**
  sidebar) and is saved.

Settings are stored permanently in `settings.json`. In **single-player** both
bindings control the same character (default: WASD *and* arrows), in
**multiplayer** one each. All games have **sound effects** (procedurally
generated, no extra files needed) that can be muted globally.

### Project structure

```
install-python.bat  Windows setup: Python 3.13 + .venv + pygame
start.bat            Launch script (Windows)
start.sh             Launch script (Linux / macOS / Git Bash)
pyinstall.bat        EXE build (Windows): bundles everything into builds\PyGameZ.exe
main.py              Tkinter UI, Pygame embedding, central game loop
game_base.py         Game base class (update/draw/handle_event) + InputEvent + helpers
settings.py          Load/save settings (sound/haptics/key bindings) (JSON)
audio.py             Procedural sound effects + gamepad rumble
menu.py              Language, pre-game (mode) and options screen (sound/controls)
highscore.py         Load/save high scores (section in mem.json)
store.py             Central save file mem.json (sections: mem, highscores, stats, achievements)
stats.py             Player statistics (plays, play time, wins, records) per game
achievements.py      Achievements: definitions, unlock logic, toast overlay
progress.py          Achievements & statistics screen (two tabs, scrollable)
prestige.py          Prestige system for Snake
competitive.py       Tuning for Snake's Competitive mode (levels, slot machine, gamble apples)
ngb.py               Visual personalization ("mods"): head color + coordinate grid + menu (mem-ngb.json)
i18n.py              Translation engine (loads lang/*.json, t("key"))
lang/
  de.json  en.json  fr.json  es.json  pt.json   Language strings (one placeholder key per text)
lamawiki/
  lamawiki.py          In-game wiki (search, categories, article renderer)
  de.json  en.json  fr.json  es.json  pt.json   Wiki content (one page per game + general pages)
games/
  snake.py  pong.py  airhockey.py  tictactoe.py  breakout.py  tetris.py
  invaders.py  asteroids.py  pacman.py  flappy.py  doodle.py
  game2048.py  minesweeper.py  sudoku.py  sudoku_gen.py
  frogger.py  memory.py  solitaire.py  cards.py  aimtrainer.py
  connect4.py  tanks.py  blackjack.py  tunnelracer.py
  labyrinth.py  maze_gen.py  reversi.py  kniffel.py  wordle.py
  trexrunner.py  dame.py  poker.py  chess.py  muehle.py
  simon.py  billiard.py  slidepuzzle.py  mastermind.py
  bubbleshooter.py  hangman.py  hangman_words.py  blockjump.py
```

The chosen language is stored in `mem.json` (in the `mem` section, next to the
`highscores` section in the same file) and loaded automatically on the next launch.

### Platform notes

The display runs **off-screen**: pygame uses the dummy video driver
(`SDL_VIDEODRIVER=dummy`), so it renders into a surface, and each frame is drawn
as an image into a Tkinter widget. There is **no native SDL window** that could
fight Tkinter over size/position. As a result the window behaves the same and
stable everywhere:

- **Windows**: the process is additionally made DPI-aware so the display stays
  sharp on scaled displays (125/150/200 %) and doesn't "shake".
- **Linux/X11 & Wayland**: works without special cases (no `SDL_WINDOWID`).
- **macOS**: works as well (previously the embedded window wasn't shown here at all).

---

### Installation Guide

Requirement: **Python 3.9+** (recommended 3.12 or 3.13) and **pygame ≥ 2.6**.

#### Windows (recommended: automatic)

1. Open the project folder and double-click **`install-python.bat`**.
   The script
   - checks whether **Python 3.13** is present, and otherwise installs it via
     **winget** (`winget install Python.Python.3.13`),
   - creates the virtual environment **`.venv`**,
   - installs **pygame** from `requirements.txt`.
2. Then launch the collection with **`start.bat`** (double-click).

> Note: if the script reports "not yet available in this window", Python was just
> installed – simply open **a new terminal/window** and run `install-python.bat`
> again. If **winget** is not available, install Python 3.13 manually from
> <https://www.python.org/downloads/> and tick **"Add python.exe to PATH"**.

#### Windows / Linux / macOS (manual)

```bash
# 1. Check Python (3.9+)
python --version

# 2. Create and activate a virtual environment
python -m venv .venv
#   Windows (cmd):        .venv\Scripts\activate
#   Windows (PowerShell): .venv\Scripts\Activate.ps1
#   Linux / macOS:        source .venv/bin/activate

# 3. Install dependencies
pip install -r requirements.txt
#   or:  pip install "pygame>=2.6" (or pygame-ce)
#                                   pip install pygame-ce
# 4. Launch
python main.py
```

#### Linux / macOS with start.sh

```bash
# Set up Python + venv as above (steps 2 and 3), then:
chmod +x start.sh      # once, if not yet executable
./start.sh
```

On Linux, install Python via the package manager if needed, e.g.
`sudo apt install python3 python3-venv python3-pip` (Debian/Ubuntu); on macOS
e.g. `brew install python`.

#### Using a different Python version

`install-python.bat` sets up Python 3.13 by default. If you prefer 3.12 (or another
version), change the line `set "PYVER=3.13"` in the file to the desired version
and the winget ID accordingly (`Python.Python.3.12`).

#### Building a standalone EXE (Windows)

```bat
pyinstall.bat         :: builds builds\PyGameZ.exe (everything in one file)
```

`pyinstall.bat` uses the `.venv` (creating it if needed), automatically
installs **PyInstaller** and bundles the complete game - Python, pygame, all
games, languages, wiki and logos - into **a single `PyGameZ.exe`** in the
**`builds\`** folder. The file runs on any Windows PC without Python
installed and can be copied freely. Settings and high scores
(`settings.json`, `mem.json`, `mem-ngb.json`) are created next to the .exe
while playing.

#### Troubleshooting

- **`pygame` not found** → is the venv activated? Repeat step 3
  (`pip install -r requirements.txt`).
- **`python` not recognized (Windows)** → Python was installed without "Add to
  PATH"; reinstall and tick the box, or use `py` instead of `python`.
- **No sound** → check "Sound" in the options; haptics only work with a controller.
- **Window/embedding on Linux** → see *Platform notes* (Wayland/XWayland).

<div align="right"><b><a href="#pygamez">↑ back to top / nach oben</a></b></div>
