# PyGameZ [[Download Latest!]](https://github.com/herrlamatv/pygamez/raw/refs/heads/master/lastest-build/PyGameZ-latest.exe) (.exe) | [web](https://download-directory.github.io/?url=https%3A%2F%2Fgithub.com%2Fherrlamatv%2Fpygamez%2Ftree%2Fmaster%2Fweb) (.zip)
If web is blocked, you can try  [web2](https://downgit.github.io/#/home?url=https://github.com/herrlamatv/pygamez/tree/master/web)

**🌐 Sprache / Language:** **🇩🇪 [Deutsch](#-deutsch)** · **🇬🇧 [English](#-english)** · **🇫🇷 [Français](other.readme.md#-francais)** · **🇪🇸 [Español](other.readme.md#-espanol)** · **🇵🇹 [Português](other.readme.md#-portugues)** · **🇵🇱 [Polski](other.readme.md#-polski)** · **🇹🇷 [Türkçe](other.readme.md#-turkce)** · **🇩🇰 [Dansk](other.readme.md#-dansk)** · **🇳🇴 [Norsk](other.readme.md#-norsk)** · **🇸🇪 [Svenska](other.readme.md#-svenska)** · **🇫🇮 [Suomi](other.readme.md#-suomi)** · **🇨🇿 [Čeština](other.readme.md#-cestina)** · **🇸🇮 [Slovenščina](other.readme.md#-slovenscina)** · **🇭🇷 [Hrvatski](other.readme.md#-hrvatski)** <br> Game Available in : German, English, French, Spanish, Portuguese, Polish, Turkish, Danish, Norwegian, Swedish, Finnish, Czech, Slovenian, Croatian (14 languages)

**📋 [Changelog](changelog.md)** (Deutsch / English)
<br>

### Note: You can download the web version directly by https://web.pygamez.com (302)

---

<a name="-deutsch"></a>

## 🇩🇪 Deutsch

Eine Desktop-Spielesammlung in Python: **Tkinter** bildet Fenster und Menü,
**Pygame** wird als Spiel-Display in das Tkinter-Fenster eingebettet. Fünfzig
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

**Latest BUILD:** [PyGameZ-latest.exe](https://github.com/herrlamatv/pygamez/raw/refs/heads/master/lastest-build/PyGameZ-latest.exe)<br>Oder Starte die [PyGameZ-2026-26-August.exe](https://github.com/herrlamatv/pygamez/releases/download/26.26.08-05.04/PyGameZ-2026-26-August.exe) (Latest Release)

#### macOS

**`start.command`** im Finder doppelklicken. Gebraucht wird nur ein Python 3.9+ mit
Tkinter, am einfachsten **Python 3.13 von [python.org](https://www.python.org/downloads/macos/)**.

#### Linux / Git Bash

```bash
./start.sh            # richtet beim ersten Start .venv + pygame ein, dann Start
```

`start.bat` / `start.sh` / `start.command` verwenden automatisch die virtuelle
Umgebung `.venv`. Fehlt sie, legen sie sie selbst an und installieren pygame-ce aus
`requirements.txt`. Eine ausführliche Schritt-für-Schritt-Anleitung steht ganz unten
unter **[Installations-Guide](#installations-guide)**.

### Die Spiele

| Spiel        | Modi            | Kurzbeschreibung |
|--------------|-----------------|------------------|
| **Snake**    | 1 / 2 Spieler   | Deluxe-Snake mit 2D- & 3D-Ansicht, Boost, 6 Spielmodi (inkl. Competitive), Goldäpfeln und Prestige |
| **Pong**     | 1 / 2 Spieler   | Klassiker gegen KI oder Spieler 2, umschaltbarer Bewegungsmodus |
| **Air Hockey** | 1 / 2 Spieler | 2D-Physik mit Impulsübertragung, Maussteuerung, KI und Power-Ups |
| **Tic-Tac-Toe** | 1 / 2 Spieler | m,n,k-Spiel auf 3x3 bis 9x9, drei KI-Stärken **oder** lokal X gegen O |
| **Breakout** | 1 Spieler       | Brick-Breaker mit Steinsorten, Power-ups, Combos und vielen Levels |
| **Tetris**   | 1 / 2 Spieler   | Moderne Guideline-Regeln (SRS, Halten, 5er-Vorschau, T-Spins): Marathon, Sprint 40, Ultra 2:00, Versus gegen die KI (3 Stärken) oder zu zweit mit Müllzeilen |
| **Invaders** | 1 Spieler       | Space Invaders: Wellen leeren, Leben schützen |
| **Asteroids** | 1 / 2 Spieler  | Trägheitsphysik, Wellen, UFOs, Power-Ups, Hyperraum - solo oder Koop-Duell |
| **Pac-Man**  | 1 Spieler       | Originalgetreuer Klon: 4 Geister-KIs, Power-Pillen, Tunnel, Früchte, Levels |
| **Flappy Bird** | 1 Spieler    | Schwerkraft-Flug durch Röhren, Münzen, Schild, Tag/Nacht, Medaillen |
| **Doodle Jump** | 1 Spieler    | Auto-Sprung nach oben, Plattform-Typen, Federn, Propeller, Monster |
| **2048**     | 1 Spieler       | Zahlen-Schiebespiel auf 3x3 bis 8x8: Klassisch, Zeitangriff und Endlos, Rückgängig, flüssige Animationen, Partien speichern & fortsetzen |
| **Minesweeper** | 1 Spieler    | Der Klassiker mit sicherem Erstklick, Chording, Smiley und Bestzeiten |
| **Sudoku**      | 1 Spieler    | 4 Varianten (Klassisch, X-Sudoku, Killer, Mini 6x6) mit je 400 Leveln, Tages-Sudoku, bis zu 3 Sterne je Level, 4 Assistenz-Modi, Rückgängig, Spielstand speichern |
| **Frogger**     | 1 Spieler    | Straße + Fluss + 5 Buchten, Bonus-Fliege, Krokodile, Zeitlimit, 3 Schwierigkeitsgrade |
| **Memory**      | 1 / 2 Spieler | Paare finden auf 4x4 bis 8x6, Flip-Animation, Solo-Wertung oder Duell |
| **Solitär**     | 1 Spieler    | 5 Varianten (Klondike, Spider, FreeCell, Pyramide, TriPeaks) mit Drag & Drop und Undo |
| **Aim Trainer** | 1 Spieler    | Chilliges 3D-Zielschießen: Maus lenkt die Kamera, 4 Modi (Präzision/Reflex/Bewegt/Chill), 3 Themen inkl. schwarzem Loch |
| **Vier gewinnt** | 1 / 2 Spieler | Der Klassiker mit Fall-Animation: 3 KI-Stärken (Minimax) oder lokales Duell |
| **Panzer-Duell** | 1 / 2 Spieler | 2D-Arena-Duell mit Ricochet-Schüssen, Power-Ups, 4 Arenen, KI mit 3 Stärken |
| **Blackjack**    | 1 Spieler    | Casino-Blackjack mit 4-Deck-Schuh, Double/Split und 3:2-Blackjack; spielt mit den Lama-Chips der gemeinsamen Lama-Bank |
| **Tunnel Racer** | 1 Spieler    | 3D-Neon-Röhrenflug: Endlos-Modus + 30 Level, Tasten- oder Maus-Steuerung, Motion Blur |
| **3D-Labyrinth** | 1 Spieler    | Ego-Raycaster (Wolfenstein-Stil) mit 50 Seed-Leveln, Orbs, Minimap - oder 2D-Draufsicht |
| **Reversi**      | 1 / 2 Spieler | Othello auf 8x8: Steine einschließen und umdrehen, 3 KI-Stärken (Minimax) oder lokales Duell |
| **Kniffel**      | 1 / 2 Spieler | Würfelklassiker mit 13 Kategorien, oberem Bonus und Kniffel; Highscore-Jagd oder 2-Spieler-Hotseat |
| **Wordle**       | 1 Spieler    | Wörter mit 4 bis 7 Buchstaben erraten: Endlos, Tageswort, Dordle und Quordle, harter Modus, Farbenblind-Palette, Statistik mit Balkendiagramm, Ergebnis teilen, echte Wortlisten in 14 Sprachen |
| **T-Rex Runner** | 1 Spieler    | Endloser Wüstenlauf: variabler Sprung, Ducken, Kakteen & Flugsaurier, Tag/Nacht-Wechsel, steigendes Tempo, 3 Schwierigkeitsgrade |
| **Dame**         | 1 / 2 Spieler | 3 Regelwerke wählbar (Deutsche 8×8, Internationale 10×10, Checkers), Schlagzwang & fliegende Dame, 3 KI-Stärken (Minimax) oder lokales Duell |
| **Poker**        | 1 Spieler    | 3 Varianten wählbar: Texas Hold'em gegen KI, 5 Card Draw und Video Poker; Setzrunden, Blinds, Lama-Chips der gemeinsamen Lama-Bank |
| **Schach**       | 1 / 2 Spieler | Vollständige Regeln, Chess960 und Schachuhr, 6 KI-Stärken, 200 Rätsel aus der Lichess-Datenbank, Rückgängig/Hinweis, Zugliste, PGN-Export oder lokales Duell |
| **Mühle**        | 1 / 2 Spieler | Nine Men's Morris mit Setz-/Zieh-/Sprungphase, Mühlen & Schlagen, fliegende Steine abschaltbar, 3 KI-Stärken oder lokales Duell |
| **Simon**        | 1 / 2 Spieler | Senso-Merkspiel: Modi Klassisch/Speed/Reverse/Gemischt + Duell, Ton aus/an/gemischt, 4/6/9 Felder, Bestwert je Modus |
| **Billard**      | 1 / 2 Spieler | 8-Ball, 9-Ball & Übungsmodus in 2D, fester 3D-Ansicht oder frei drehbarer 3D-Kamera; weiche Physik, Zielhilfe, KI mit 3 Stärken |
| **Schiebepuzzle** | 1 Spieler    | 15-Puzzle in 3x3/4x4/5x5: durchnummerierte Kacheln in die Lücke schieben, Klick- oder Pfeilsteuerung, Punkte nach Zügen & Zeit |
| **Mastermind**    | 1 Spieler    | Geheimen Farbcode knacken (3 Modi: 4×6, klassisch, 5×8), schwarze/weiße Rückmeldungs-Pins, Endlos-Streak als Highscore |
| **Bubble Shooter** | 1 Spieler   | Puzzle-Bobble-Klon: gleiche Farben zu Dreiergruppen schießen, Wandreflexion, herabfallende Cluster, 3 Schwierigkeitsgrade |
| **Galgenmännchen** | 1 Spieler   | Wort erraten, bevor der Galgen voll ist; Bildschirmtastatur, Wortlisten je Sprache, 3 Längen-Modi, Endlos-Streak |
| **Block Jump**  | 1 Spieler       | 3D-Jump'n'Run im Minecraft-Stil: texturierte Voxel-Welt, Steve-Figur, Leitern, Zäune & Schleimblöcke, Ego-/Verfolgerkamera, seed-generierte Parkour-Level |
| **Tower Defense** | 1 Spieler     | Endlose Wellen auf 4 Karten abwehren: bis zu 11 Turmtypen mit Ausbau, Verkauf & A/B-Spezialisierung, Bosse, 3 Modi, Aktiv-Fähigkeiten |
| **Minigolf**    | 1 / 2 Spieler   | 360 Bahnen in 40 Kursen (18 handgebaut, 342 seed-erzeugt): Sand, Rampen, Wasser, Gummipuffer, Windmühlen & Wanderblöcke; Scorekarte mit Par und Hole-in-One-Bonus; **eigener Bahn-Editor** mit 15 Objekttypen, 12 Vorlagen und Teilen als `.lamapgzmap` |
| **Pinball**     | 1 / 2 Spieler   | Flipperautomat mit 3 Tischen: Pop-Bumper, Slingshots, Drop-Targets, L-A-M-A-Bahnen, Multiball mit Jackpot, Ball-Save, Nudge & Tilt |
| **Bowling**     | 1 / 2 Spieler   | 10 Frames mit offizieller Strike-/Spare-Wertung, echter Pin-Physik, Hook-Effet und perspektivischer Bahnansicht, 3 Schwierigkeiten |
| **Crossy Road** | 1 Spieler       | Endlos über Wiesen, Straßen, Flüsse und Gleise hüpfen im isometrischen Voxel-Look: Tag/Nacht-Wechsel, Adler, 10 kaufbare Figuren, Tagesstrecke |
| **Geometry Dash** | 1 Spieler     | Rhythmus-Plattformer mit Würfel, Schiff, Ball, UFO und Welle: 16 Level von Leicht bis Dämon mit je 3 geheimen Münzen, dazu eine Bibliothek aus **80 weiteren Leveln** in 10 Kategorien mit Suche, Übungsmodus, Soundtrack je Level; **Level-Editor** mit 12 Vorlagen und Teilen als `.lamapgzlevel` |
| **Schiffe versenken** | 1 / 2 Spieler | 10x10-Seeschlacht: Flotte per Drag & Drop aufstellen, 3 Regel-Schalter (Berühren, Salve, Nochmal schießen), KI mit 3 Stärken oder lokales Duell mit Übergabe-Bildschirm |
| **Casino**      | 1 Spieler       | Europäisches Roulette mit allen klassischen Wetten und Lama-Slot (5 Walzen, 10 Linien, Wild, Freispiele); ein Lama-Chip-Konto mit Blackjack und Poker |
| **Lama Crossing** | 1 Spieler       | Mut-Spiel mit Lama-Chips: Spur für Spur über Straße, Hängebrücke, Fluss oder Eissee, Lücken im Verkehr abpassen, nach jeder Spur auszahlen; 4 Schwierigkeiten bis ×31.784, Quote ~97 % |
| **Lama Mines**  | 1 Spieler       | Mut-Spiel mit Lama-Chips: Edelsteine aufdecken, Minen meiden - Felder 4x4 bis 7x7 mit 1 bis 48 Minen, jeder Edelstein hebt den Multiplikator bis ×100.000, jederzeit auszahlen; Auto-Modus, Fairness-Check per SHA-256, Statistik, Quote ~97 % |
| **Lama Plinko** | 1 Spieler       | Goldene Wollknäuel fallen durch 8 bis 16 Stiftreihen in Multiplikator-Fächer: 3 Risikostufen bis ×1000, bis zu 50 Kugeln gleichzeitig, Dauerfeuer und Auto-Modus; Fairness-Check je Kugel, Treffer-Verteilung, Quote ~97 % |
| **Lama Crash**  | 1 Spieler       | Crash-Spiel mit Lama-Chips: Das Lama steigt im Ballon, der Multiplikator wächst bis ×100.000 - rechtzeitig aussteigen, bevor er platzt; zwei Einsätze mit Auto-Ziel und Hälfte, 6 Höhenwelten bis zum Mond, Spannungsmusik, Auto-Modus, Fairness-Check, Statistik, Quote ~97 % |

**Mehrspieler (2 Spieler lokal)** gibt es für **Snake**, **Pong**, **Air Hockey**,
**Tic-Tac-Toe**, **Tetris (Versus)**, **Asteroids (Koop-Duell)**, **Memory (Duell)**,
**Vier gewinnt**, **Panzer-Duell**, **Reversi**, **Kniffel**, **Dame**, **Schach**,
**Mühle**, **Simon (Duell)**, **Billard**, **Minigolf**, **Pinball**, **Bowling**
und **Schiffe versenken** (mit verdecktem Übergabe-Bildschirm) - zusammen 20 Spiele.
Der Modus wird direkt im Vorspiel-Screen (*Einzelspieler / Mehrspieler*) gewählt;
Tetris bietet zusätzlich **Versus gegen die KI**. Die Web-Version ist reiner
Einzelspieler.

#### Feature-Details je Spiel

**Snake**
- **NEU - Replay der Runde**: Jeder Lauf wird mitgeschnitten - am Game Over
  zeigt **P** die Wiederholung (immer von oben, auch wenn du in 3D gespielt
  hast), **S** legt sie ins Archiv, **E** teilt sie als `.lamapgzreplay`-Datei.
  Aufgezeichnet werden Körper, Äpfel, Gold- und Spezialäpfel, Punkte und
  Ausdauer - in Kapiteln von zehn Sekunden, damit das Spulen flott bleibt.
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
  Alles wird in `mem.json` (Abschnitt `ngb`, je Profil) gespeichert; die visuelle Personalisierung läuft über
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
- **NEU - Replay der Partie**: Jede Runde wird mitgeschnitten - nach dem
  Ergebnis zeigt **P** die Wiederholung mit Feld, Vorschau, Hold, Müllbalken und
  Statistik (im Versus beide Felder), **S** legt sie ins Archiv, **E** teilt sie
  als `.lamapgzreplay`-Datei.
- **Moderne Guideline-Regeln**: 10x20-Feld, Steine im **7er-Beutel**,
  **SRS-Drehsystem** mit echten Wall Kicks (auch für den I-Stein), Drehen in
  beide Richtungen, **Halten** (einmal pro Stein), **Vorschau auf 5 Steine**,
  Landeschatten und **Lock Delay** (0,5 s, höchstens 15 Resets).
- **Drei Modi** im Vorspiel-Screen: *Solo*, *Versus KI* und *2 Spieler*. Solo
  bietet im Setup **Marathon** (Startlevel 1-15, zählt als Highscore),
  **Sprint 40 Zeilen** (Bestzeit) und **Ultra 2 Minuten** (Bestwert); die
  Bestwerte liegen im Abschnitt `tetris` von `mem.json`.
- **Wertung nach Guideline**: Single bis Tetris, **T-Spins** (voll und Mini),
  **Back-to-Back** (x1,5), **Combos** und **Perfect Clear** - mit Einblendungen,
  Lösch-Animation, Hard-Drop-Spur, Partikeln und Level-Up-Effekt.
- **Versus mit Müllzeilen**: Abgeräumte Zeilen schicken Müll zum Gegner (Tetris
  = 4, T-Spin Double = 4 …), eingehender Müll wird im Warnbalken angekündigt
  und mit eigenen Angriffen **verrechnet**; beide Felder bekommen dieselbe
  Steinfolge. Die **KI** gibt es in 3 Stärken, alle 40 Sekunden wird es
  schneller.
- **Steuerung**: Links/Rechts mit eigenem **DAS/ARR** (im Setup einstellbar),
  Hoch = rechts drehen, Runter = Soft Drop, Aktion = Hard Drop; **C**/Shift =
  Halten, **Z**/**Y** = links drehen, **X** = rechts drehen. Zu zweit hält P1 mit
  **Q** und dreht links mit **E**, P2 mit **Shift rechts** / **Strg rechts**.
  Nach dem Spielende: **R** = nochmal, **S** = Setup.

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

**2048**
- **Eigener Setup-Screen** mit Brettgrößen **3x3 bis 8x8** und drei Modi:
  *Klassisch* (Ziel 2048, danach „Weiterspielen?"), *Zeitangriff* (3 Minuten,
  die Uhr startet mit dem ersten Zug) und *Endlos*.
- **Flüssige Animationen**: Kacheln gleiten, verschmelzen mit Pop und wachsen
  hinein; Punkte-Popups, Funken ab 128, Druckwelle ab 2048 und neue Farben bis
  131072. Eingaben während einer Animation werden gepuffert.
- **Rückgängig** (aus / 3 pro Partie / unbegrenzt, Taste **U** oder Backspace) -
  wer es benutzt, spielt ohne Highscore und ohne Kachel-Erfolge.
- **Partien speichern & fortsetzen**: Die laufende Partie wird je Größe und Modus
  automatisch gesichert; Bestwerte und größte Kachel je Größe/Modus stehen im
  Abschnitt `g2048` von `mem.json`.
- Steuerung: Pfeile/WASD oder **Wischen** mit Maus/Touchpad, **R**/**N** = neue
  Partie, **Tab** = Setup. Der Highscore zählt nur bei **4x4 Klassisch** ohne
  Rückgängig.

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
- **4 Varianten** mit je **400 Leveln** (4 Stufen x 100): *Klassisch* (die
  bekannten Seed-Level - gelöste Level bleiben abgehakt), *X-Sudoku* (beide
  Diagonalen enthalten jede Ziffer genau einmal), *Killer* (gestrichelte Käfige
  mit Summe; 400 vorab erzeugte Level) und *Mini 6x6*. Alle Puzzles sind
  **eindeutig lösbar** - Level 12 von "Schwer" ist auf jedem PC dasselbe Puzzle.
- **Tages-Sudoku**: ein Rätsel pro Tag für alle, am PC und im Browser identisch;
  die Stufe hängt vom Wochentag ab (Montag Leicht bis Samstag Experte), wer
  täglich löst, baut eine Serie auf.
- **Bis zu 3 Sterne je Level** (gelöst · fehlerfrei ohne Tipps · zusätzlich unter
  der Zielzeit) und **Bestzeit** in der Levelauswahl; **angefangene Rätsel**
  werden automatisch gespeichert und beim nächsten Start fortgesetzt.
- **4 Spielmodi** (Auswahl vor dem Start) mit Punkte-Multiplikator:
  **Klassisch** (x2,0 - keine Hilfen), **Notizen** (x1,5 - + Bleistift-
  Notizen und Kandidaten-Automatik), **Komfort** (x1,0 - + Fehler rot,
  Konflikte und falsche Käfigsummen markiert, korrekte Eingaben rasten ein),
  **Assistent** (x0,7 - + Tipp-Funktion, max. 3). Mit aktivem
  **3-Fehler-Limit** (Option im Setup) ist beim dritten Fehler Schluss.
- Steuerung: Pfeile/WASD = Zelle, **1-9** = Ziffer (auch Ziffernblock),
  **0/Entf/Rechtsklick** = radieren, **U**/**Z** = rückgängig, **Y** =
  wiederherstellen, **N** = Notizen, **C** = Kandidaten automatisch eintragen,
  **H** = Tipp, **M** = Farbmarker, **R** = Level neu, **Q** = Levelwahl.
  Eingabe „Ziffer zuerst" (Setup, **I**) und ein **Restziffer-Zähler** unter
  jeder Ziffer; komplett mit der Maus spielbar. Nach Spielende zeigt **A** die
  komplette **Lösung**.
- Punkte = (Basis von Variante und Stufe - Zeit - Fehler - Tipps) x
  Modus-Multiplikator; alle Varianten und das Tages-Sudoku zählen für den
  Highscore.

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
- **Lama-Chips**: Blackjack spielt mit dem Konto der **Lama-Bank**, das es sich
  mit Poker und Casino teilt (Start 1000, dauerhaft in `mem.json`). Der Einsatz
  wird beim Geben sofort abgebucht; unter 10 Chips gibt es per Enter einen
  **Bank-Kredit** auf 1000.
- **Highscore** = Höchststand der eigenen **Blackjack-Bilanz** (1000 plus alles,
  was beim Blackjack gewonnen und verloren wurde) - Gewinne am Roulette, am Slot
  oder beim Poker zählen hier nicht, Kredite auch nicht.
- Bedienung per Chip-Buttons und Tasten (**H**it/**S**tand/**D**ouble/Split
  **X**, **1-4** = Einsatz, Backspace = Einsatz zurück, Enter = Geben) mit
  Karten-Animationen; die verdeckte Dealer-Karte dreht sich beim Aufdecken
  jetzt wirklich um.

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
- Errate das gesuchte Wort; farbige Rückmeldung (grün/gelb/grau) mit korrekter
  **Doppelbuchstaben-Zählung** und mitfärbender Bildschirmtastatur (QWERTZ für
  Deutsch, Tschechisch, Slowenisch und Kroatisch, AZERTY für Französisch, sonst
  QWERTY).
- **Vier Modi**: *Endlos* (ein Wort nach dem anderen mit je 6 Versuchen; jedes
  gelöste Wort bringt Punkte, das erste ungelöste beendet die Partie), *Tageswort*
  (ein Wort pro Tag je Sprache und Länge - am PC und im Browser dasselbe - mit
  Countdown und Serie; angefangene Tageswörter werden gespeichert), *Dordle* (2
  Wörter gleichzeitig in 7 Versuchen) und *Quordle* (4 Wörter in 9 Versuchen, die
  Tasten zeigen die Farben aller Bretter).
- **Setup** vor jeder Partie: **Wortlänge 4 bis 7**, **harter Modus** (gefundene
  Hinweise müssen weiterverwendet werden - grüne Buchstaben bleiben an ihrem
  Platz, gelbe müssen wieder vorkommen) und **Farbenblind-Palette** (Orange/Blau);
  daneben steht die Statistik.
- **Echte Wortlisten in allen 14 Sprachen** (Ordner `woordlistz/`, nur A-Z), für
  jede Länge eigene: allein bei 5 Buchstaben **33.847 Lösungswörter** und
  **213.375 erlaubte Rateworte**, über alle vier Längen rund 134.000
  Lösungswörter. Gesucht sind geläufige Wörter ohne Namen, englische Einsprengsel
  und anstößige Wörter; jeder Rateversuch wird gegen die Liste geprüft - was nicht
  darin steht, wird abgelehnt und die Zeile wackelt kurz.
- **Statistik** je Sprache, Länge und Modus: Partien, Gewinnquote, aktuelle und
  beste Serie und die **Versuchsverteilung als Balkendiagramm** (Abschnitt
  `wordle` von `mem.json`). **Ergebnis teilen** (**C**) legt ein Emoji-Raster ohne
  die Lösung in die Zwischenablage.
- Der Highscore zählt nur *Endlos* mit 5 Buchstaben, die anderen Längen haben
  eigene Bestwerte. Erfolge **Hellseher** (höchstens 2 Versuche),
  **Wortgewohnheit** (7 Tageswörter in Folge) und **Vierfach-Genie** (Quordle
  gelöst).

**Poker**
- **3 Varianten** im Vorspiel-Screen: **Texas Hold'em** gegen 1-3 KI-Gegner mit
  Dealer-Button, Blinds und vier Setzrunden, **5 Card Draw** (Heads-up gegen die
  KI, einmal Karten tauschen) und **Video Poker** (*Jacks or Better*, solo gegen
  die Auszahlungstabelle).
- Aktionen per Buttons oder Tasten: **F** = Fold, **C** = Check/Call, **R** =
  Raise, **A** = All-In; Karten halten/tauschen per Klick oder **1-5**, **Enter**
  zieht bzw. gibt die nächste Hand.
- **Lama-Chips** der gemeinsamen **Lama-Bank**: Zu Beginn einer Hand liegt das
  Konto als Stapel am Tisch, was in den Pot geht, ist sofort abgebucht - wer den
  Tisch mitten in der Hand verlässt, verliert nur seinen Pot-Anteil. Pleite
  (unter dem Big Blind von 20, im Video Poker unter 10) = Bank-Kredit auf 1000.
- **Highscore** = Höchststand der **Poker-Bilanz** (1000 plus alle
  Poker-Gewinne und -Verluste); auch der Erfolg **Chipleader** zählt nur die
  Poker-Bilanz.

**Schach**
- **Vollständiges Schach**: alle Figurenzüge inkl. **Rochade**, **En Passant**
  und **Bauernumwandlung** (Figur wählbar); **Schach, Schachmatt und Patt** sowie
  Remis durch **50-Züge-Regel**, **dreifache Stellungswiederholung**,
  **ungenügendes Material** oder Einigung.
- **Drei Modi**: *Partie* gegen die KI, *2 Spieler* am selben Rechner (das Brett
  dreht sich auf Wunsch nach jedem Zug) und **Rätsel**.
- **Stärkere, ruckelfreie KI** in 6 Stufen von *Anfänger* bis *Meister*:
  iterative Vertiefung, Transpositionstabelle, Ruhesuche, Eröffnungsbuch und
  eine Bewertung mit Mobilität, Bauernstruktur und Königssicherheit. Die KI
  rechnet in kleinen Häppchen pro Bild - das Spiel ruckelt nie.
- **Setup**: Farbwahl, **Schachuhr** (ohne, 1+0, 3+2, 5+0, 10+5) und
  **Chess960** (alle 960 Grundstellungen, die Nummer steht über der Zugliste).
- **Seitenleiste** mit Uhren, geschlagenen Figuren, Materialbilanz und
  scrollbarer **Zugliste (SAN)**; Drag & Drop, gleitende Figuren, Koordinaten.
  Tasten: **U** = Rückgängig, **H** = Hinweis-Pfeil, **O** = Remis anbieten,
  **X** = Aufgeben, **F** = Brett drehen, nach der Partie **P** = **PGN-Export**.
- **Rätsel**: 200 Aufgaben in 5 Stufen (Matt in 1/2/3, Taktik I/II) aus der
  freien **Lichess-Rätseldatenbank (CC0)**, mit der eigenen Engine geprüft; bei
  Matt-Aufgaben zählt jeder mattsetzende Zug. Der Fortschritt steht im Abschnitt
  `chess` von `mem.json`.
- Rückgängig und Hinweis machen eine Partie „unterstützt": Der Highscore zählt
  nur Siege gegen die KI ohne Hilfe (je Sitzung).

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
- **NEU - Replay der Partie**: Jeder Stoß wird als eigene Sequenz aufgezeichnet
  - mit Zielwinkel, Stärke und der Bahn aller Kugeln. Am Partieende zeigt **P**
  die Wiederholung (im Übungsmodus jederzeit zwischen zwei Stößen), **S** legt
  sie ins Archiv, **E** teilt sie als `.lamapgzreplay`-Datei.
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
  schlägt, rechte Maustaste halten sperrt die Kraft (alternativ Pfeile +
  Leertaste). **R** bricht einen geladenen Schlag ab, ohne zu putten. **G**
  blendet die Ziellinie um, **Z** das Autoziel, **P** das Aufnehmen. **F**
  setzt die laufende Bahn zurück: Schläge auf 0, Ball zurück aufs Tee -
  gleiche Bahn, gleicher Kurs.
- **Stärke-Sperre (rechte Maustaste halten)**: friert den Ladebalken genau da
  ein, wo er beim Drücken stand - golden, mit Prozentzahl, Schloss und ruhig
  pulsendem Ring um den Ball. So wartet man mit fertig geladenem Schlag auf die
  Lücke in der Windmühle oder auf den Wanderblock und puttet im richtigen
  Moment. Loslassen lädt weiter; die gesperrte Kraft übersteht sogar den
  Schlag, der nächste Linksklick puttet exakt mit diesem Wert - auch eine mit
  Pfeil hoch/runter eingestellte Stärke lässt sich so festnageln.
- **Weiter statt Wiederholung**: Am Rundenende führt der Knopf **Weiter** zum
  nächsten Kurs (*Classic* → *Pro* → *Tour 1* → *Tour 2* …), sodass nie wieder
  derselbe Neuner-Satz kommt; daneben stehen **Nochmal** (gleicher Kurs) und
  **Setup**. Tasten: Enter = weiter, R = nochmal, S = Setup.
- **Scorekarte** rechts mit Par und Schlägen je Bahn; zu zweit spielt jeder
  dieselbe Bahn nacheinander. Punkte: 600 je Bahn, ±300 je Schlag unter/über
  Par, **500 extra für ein Hole-in-One**. Die niedrigste Schlagzahl je Kurs
  steht im Abschnitt `minigolf` von `mem.json`.
- **Aufnehmen ist abschaltbar**: Standardmäßig endet eine Bahn nach acht
  Schlägen und wird mit dem Mindestwert gewertet. Wer lieber bis zum Einlochen
  weiterspielt, stellt *Aufnehmen* im Setup auf AUS (oder drückt **P**).
- **Autoziel ist abschaltbar**: Standardmäßig dreht sich der Schläger vor jedem
  Schlag von selbst zum Loch. Wer lieber jede Bahn selbst anvisiert, stellt
  *Autoziel* im Setup auf AUS (oder drückt **Z**) - dann bleibt die zuletzt
  gewählte Richtung stehen, und am Tee einer neuen Bahn zeigt der Schläger
  neutral bahnaufwärts.
- **Replay der Runde**: Am Rundenende zeigt **P** (oder der Knopf **Replay**)
  die komplette Runde noch einmal - Schlag für Schlag, mit Bahn, HUD und
  Scorekarte. Mit **S** wandert sie ins Archiv (Sidebar-Knopf **Replays**).
- **Eigene Bahnen bauen und teilen**: Der Reiter **MAPS** im
  Vorbereitungsbildschirm führt zur eigenen Sammlung - **Neu** öffnet den
  Bahn-Editor. Jede Bahn bekommt einen Namen und eine **id** (klein, ohne
  Leerzeichen); die id ist zugleich der vorgeschlagene Dateiname beim Teilen.
  Zu den sieben klassischen Hindernissen kommen **acht neue**: Rohr (versetzt
  den Ball ans andere Ende), Eis, Klebefeld, Booster, Magnet, Einbahn-Tor,
  Drehscheibe und Sprungrampe. Die Bahngröße lässt sich frei einstellen
  (60x80 bis 160x240), **12 Vorlagen** geben einen Startpunkt, Undo/Redo und
  **Test** gehören dazu. **Teilen** schreibt genau eine Bahn als
  `.lamapgzmap`-Datei - per Speichern-Dialog oder direkt in den
  Downloads-Ordner, mit deinem Namen als Ersteller. **Import** liest sie wieder
  ein und weicht bei belegter id automatisch auf `-2` aus. Map-Name, id und
  Ersteller-Name laufen dabei immer durch einen **Wortfilter über alle 14
  Sprachen**. Gespielt wird eine Bahn einzeln über **Spielen** oder die ganze
  Sammlung über die fünfte Kurswahl **Eigene**.

**Pinball**
- **NEU - Replay der Partie**: Je Kugel eine Sequenz - Ballbahnen (auch im
  Multiball), Flipperstellung, Drop-Targets, Bahnen, Multiplikator und
  Punktestand. Am Partieende zeigt **P** die Wiederholung, **S** legt sie ins
  Archiv, **E** teilt sie als `.lamapgzreplay`-Datei.
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
- **Replay der Partie**: Am Ende zeigt **P** alle Würfe noch einmal - Ball und
  Pins genau wie geworfen; **S** legt die Aufnahme ins Archiv (Sidebar-Knopf
  **Replays**).

**Crossy Road**
- **Endloses Hüpfen** über Wiesen (Bäume und Steine versperren den Weg), Straßen
  mit Autos und Lastern, Flüsse mit Stämmen und Seerosen und **Gleise**, auf
  denen nach Warnlicht und Klingel ein Zug heranrast - später warten Bahnhöfe
  mit bis zu 5 Gleisen. Die Strecke entsteht Reihe für Reihe, hat immer einen
  begehbaren Weg, und Tempo und Verkehr nehmen zu.
- **Isometrischer Voxel-Look**: Figuren, Fahrzeuge und Bäume aus schattierten
  Quadern (je Kachelgröße vorgerendert), weich folgende Kamera, Squash & Stretch
  beim Hüpfen, Wassersplash, Plattdrück-Animation, Federn und Münzglitzern; ab
  Reihe 50 **Tag/Nacht-Wechsel** mit Scheinwerfern.
- **Der Adler**: Die Kamera kriecht vorwärts - wer zu lange trödelt oder mehr als
  drei Reihen zurückgeht, wird geholt (ein roter Rand warnt vorher). Wer auf
  einem Stamm über den Bildrand treibt, ist ebenfalls raus.
- **Münzen & Figuren**: Gesammelte Münzen (Riesenmünze = 5) bleiben gespeichert
  und kaufen im Reiter **Figuren** neue Spielfiguren - Frosch, Schwein, Pinguin,
  Katze, Fuchs, Lama, Roboter, Geist und Einhorn (25 bis 250 Münzen); das Huhn
  ist von Anfang an dabei.
- **Modi**: *Endlos* (Punkte = weiteste Reihe, zählt als Highscore) und
  *Tagesstrecke* (heute für alle gleich, auch im Browser, mit eigenem
  Tagesbestwert). Steuerung: Pfeile/WASD, Leertaste/Enter/Klick = vorwärts; im
  Setup **H** = Schatten, **N** = Tag/Nacht. Münzen, Figuren und Tagesbestwert
  stehen im Abschnitt `crossy` von `mem.json`.

**Geometry Dash**
- **Rhythmus-Plattformer**: Die Figur rast von selbst nach rechts, du entscheidest
  nur, wann gesprungen oder geflogen wird. **Fünf Formen** - Würfel, Schiff, Ball,
  UFO und Welle -, dazu Form-, Schwerkraft- und Tempo-Portale (0,5x bis 3x),
  gelbe/pinke/blaue **Pads und Orbs**, Halbblöcke, Stacheln, Gruben und
  Farb-Trigger.
- **16 eingebaute Level** von *Leicht* bis *Dämon* („Lama Inferno", „Lama
  Eclipse") mit je **3 geheimen Münzen**. Jedes Level ist nachweislich schaffbar:
  Ein Solver hat es beim Bauen mit dem echten Spiel-Code gelöst - samt allen
  Münzen und sogar mit 1/240 Sekunde Versatz.
- **Reiter „Weitere": 80 Level in 10 Kategorien** (Einstieg, Rhythmus, Münzen,
  Schiff, UFO, Ball, Schwerkraft, Tempo, Welle, Dämon) mit **Suchfeld**,
  Zufallsknopf und eigenem Fortschritt je Level. Auch sie sind alle vom Solver
  bewiesen - Ziel und alle drei Münzen. Sterne und Erfolge bleiben der Kampagne.
- **Präzise Physik**: Festkomma-Rechnung im festen 240-Hz-Takt; jeder Druck wirkt
  genau in dem Schritt, in dem er passiert ist - bei jeder Bildrate gleich und im
  Browser bitgleich.
- **Übungsmodus** (**P**) mit automatischen und eigenen Checkpoints (**Z** setzen,
  **X** löschen), Versuchszähler, Fortschrittsbalken, Explosionen und sofortigem
  Neustart (**R**). Jedes Level hat einen **eigenen Soundtrack** - Hintergrund,
  Boden und Orbs pulsieren im Takt (Musik mit **M** abschaltbar).
- **Sterne & Münzen**: Wer ein Level im Normalmodus schafft, bekommt seine Sterne,
  jede Münze ist einen weiteren Stern wert; der Highscore sind die **Sterne
  gesamt** (höchstens 135). Bestwerte je Level, Münzen, Versuche und Sprünge
  stehen im Abschnitt `geodash` von `mem.json`.
- **Level-Editor** im Reiter **LEVELS**: Raster-Leinwand, Palette mit 6 Gruppen
  (Blöcke, Gefahren, Pads & Orbs, Portale, Tempo, Extras), Drehen, Undo/Redo,
  Übersichtsleiste, **Test ab Start oder ab hier** und Level-Einstellungen
  (Starttempo, Startform, Musikstil, BPM, Farben). Das Häkchen **„verifiziert"**
  gibt es erst, wenn du dein Level selbst geschafft hast. **Teilen** schreibt eine
  `.lamapgzlevel`-Datei, **Import** liest sie wieder ein; gespeichert wird in
  `ugc.json` neben den eigenen Minigolf-Bahnen.
- **Vorlagen**: „Neu" fragt zuerst nach einer von **12 Vorlagen** (Leer,
  Einstieg, Sprünge, Pads & Orbs, Schiff, Welle, Ball, UFO, Schwerkraft, Tempo,
  Münzen, Dämon). Sie bringen Objekte, Farben, Musik und - weil vom Solver
  durchgespielt - das Häkchen „verifiziert" mit. Im Reiter **Weitere** kopiert
  **„Als Vorlage"** jedes fertige Level in die eigenen zum Umbauen.

**Schiffe versenken**
- **10x10-Seeschlacht** mit Flugzeugträger (5 Felder), Schlachtschiff (4),
  Kreuzer (3), U-Boot (3) und Zerstörer (2) - wer zuerst alle gegnerischen
  Schiffe versenkt, gewinnt.
- **Flotte aufstellen** per Drag & Drop aus dem Dock: **R** oder Rechtsklick
  dreht, die Vorschau leuchtet grün oder rot, **X** stellt zufällig auf, **C**
  räumt das Brett; die letzte Aufstellung wird wieder vorgeschlagen.
- **Regeln im Setup** (gespeichert): *Schiffe dürfen sich berühren*, *Salve* (so
  viele Schüsse pro Zug, wie eigene Schiffe schwimmen) und *Nochmal schießen nach
  Treffer*.
- **KI mit 3 Stärken**: Leicht schießt zufällig, Mittel jagt Treffer systematisch
  nach, Schwer rechnet eine **Wahrscheinlichkeitskarte** mit Schachbrett-Parität
  (im Schnitt rund 70 / 60 / 45 Schüsse für eine ganze Flotte). Oder **2 Spieler**
  am selben Rechner - ein **Übergabe-Bildschirm** verdeckt vor jedem Zug beide
  Flotten.
- **Optik**: Radar-Sweep, animierte Wellen, Granaten im Bogen, Einschlag-Splash,
  Explosionen mit Rauch und brennenden Feldern, „VERSENKT!"-Enthüllung und am
  Ende eine Statistik mit Schüssen, Treffern und Trefferquote. Der Highscore
  zählt die **Siege gegen die KI** einer Sitzung.

**Casino**
- **Roulette** (europäisch, 37 Fächer): alle klassischen Wetten per Klick auf
  Zahl, Kante oder Ecke - **Plein** (35:1), Cheval, Transversale, Carré, Sixain,
  Kolonne, Dutzend, Rot/Schwarz, Gerade/Ungerade und Manque/Passe. Chipwerte
  1/5/25/100/500, Rechtsklick nimmt Chips zurück; **Drehen**, **Wiederholen**
  (**R**), **Verdoppeln** (**D**) und **Löschen**. Die Kugel läuft spiralförmig
  in das vorher gezogene Fach, oben stehen die letzten 12 Zahlen.
- **Lama-Slot**: 5 Walzen x 3 Reihen, **10 Gewinnlinien**, **Lama = Wild**,
  **Goldmünzen = Scatter** mit 10 Freispielen und doppelten Gewinnen, Einsatz je
  Linie 1/2/5/10, **Auto-Spin** (10/25), **Turbo** und Gewinntabelle. Die
  **Auszahlungsquote liegt bei 96,1 %** - exakt aus den Walzenstreifen berechnet.
- **Lama-Bank**: Casino, Blackjack, Poker, Lama Crossing, Lama Mines, Lama
  Plinko und Lama Crash teilen sich ein Konto mit **Lama-Chips** (Start 1000,
  Abschnitt `casino` in `mem.json`); alte Chipstände werden automatisch
  übernommen. Einsätze werden sofort abgebucht, jedes Spiel führt seine eigene
  Bilanz für den Highscore, und bei Pleite gibt es einen **Bank-Kredit** auf
  1000.
- Konfetti, Münzregen, Big-/Mega-/Jackpot-Banner und Gewinnlinien-Animationen;
  Erfolge **Volltreffer** (Plein beim Roulette) und **Lama-Jackpot** (5 Lamas
  auf einer Linie).

**Lama Crossing**
- **Mut-Spiel mit Lama-Chips**: Einsatz wählen (10 bis 10000), **LOS** drücken -
  das Lama überquert Spur für Spur eine Straße, eine Schlucht, einen Fluss oder
  einen Eissee. Jede geschaffte Spur hebt den **Multiplikator**, wer erwischt
  wird, verliert den Einsatz. Nach jeder Spur heißt es **WEITER** oder
  **AUSZAHLEN**; am anderen Ende wird automatisch ausgezahlt - mit Konfetti.
- **Hindernisse & Timing**: Auf der Straße fahren Autos und Lastwagen jede Spur
  entlang, auf der Hängebrücke stürzen Felsbrocken durch die Brücke, im Fluss
  treibt Treibholz heran und auf dem Eissee rutschen Eisblöcke. Wer losgeht,
  während die nächste Spur belegt ist, wird überfahren oder getroffen und liegt
  **platt** da - also auf eine Lücke warten. Die Spur, auf der das Lama steht,
  ist geschützt (auf der Straße senkt sich eine Schranke und die Autos halten
  an, sonst verschwinden die Hindernisse dort). Höhere Stufen bringen dichteren,
  schnelleren Verkehr.
- **4 Schwierigkeiten**: Leicht (22 Spuren, Risiko 1/12, bis ×6,57), Mittel (20,
  1/6, bis ×37,18), Schwer (18, 3/10, bis ×595,67) und Extrem (15, 1/2, bis
  ×31.784,96 - jede Spur verdoppelt). Das Zufallsrisiko je Spur kommt zum
  Verkehr dazu: ein plötzlicher **Raser** auf der Straße, eine brechende Planke,
  ein wegrutschender Stein oder einbrechendes Eis. Die Multiplikatoren stehen
  als feste Ganzzahl-Tabellen in `lamacrossing_logic.py` und sind so berechnet,
  dass **jede Auszahl-Strategie bei sauberem Timing rund 97 % zurückgibt** -
  egal, wann man aussteigt.
- **4 Welten** (Knöpfe oben links, Taste **W**): die **Straße** von oben mit
  Gehwegen und Zebrastreifen-Stücken (Standard), die **Hängebrücke** über der
  Schlucht, **Trittsteine im Fluss** und **Eisschollen auf dem gefrorenen See**.
  Das Lama trägt einen Schal in der Akzentfarbe des Spiels; oben rechts steht
  der Verlauf der letzten 12 Runden.
- **Lama-Bank**: dasselbe Konto wie Blackjack, Poker und Casino. Der Highscore
  ist der Höchststand von 1000 + Lama-Crossing-Bilanz, unter 10 Chips gibt es
  einen **Bank-Kredit** auf 1000. Wer mitten in der Überquerung geht, verliert
  nichts: Einsatz, Schwierigkeit und Spur werden gespeichert, der nächste Start
  macht genau dort weiter.
- Bedienung: **Leertaste**/**→**/Klick auf die nächste Spur = LOS bzw. weiter
  (sobald die Spur frei ist), **Enter**/**C** = auszahlen,
  **↑**/**↓**/**+**/**-**/Mausrad = Einsatz, **1-4** = Schwierigkeit, **W** =
  Welt; Welt, Schwierigkeit und letzter Einsatz stehen im Abschnitt
  `lamacrossing` der `settings.json`. Erfolge **Am anderen Ufer**,
  **Nervenstark** (Auszahlung ab ×10) und **Drahtseilakt** (auf Extrem ab ×100
  auszahlen) sowie ein Meilenstein von 3000 und Meister 10000.

**Lama Mines**
- **Mut-Spiel mit Lama-Chips**: Einsatz wählen (10 bis 10000), **SPIELEN**
  drücken - unter den verdeckten Steinquadern liegen Edelsteine und Minen.
  Jeder aufgedeckte **Edelstein** hebt den **Multiplikator**, eine **Mine**
  kostet den Einsatz. Ab dem ersten Edelstein darf jederzeit **AUSGEZAHLT**
  werden; wer alle Edelsteine findet („ALLES MEINS!") oder den Höchstgewinn
  erreicht, bekommt automatisch ausgezahlt.
- **4 Feldgrößen** (Tasten **1-4**): 4x4, 5x5 (Standard), 6x6 und 7x7, dazu
  **1 bis N−1 Minen** (**←**/**→** oder sechs Schnellwahl-Knöpfe, Standard 3) -
  auf 7x7 also bis zu 48 Minen und genau ein Edelstein.
- **Multiplikator nach k Edelsteinen** (in Hundertsteln, N Felder, M Minen):
  `m_k = max(⌊97 · C(N, M) / C(N−k, M)⌋, m_(k−1) + 1)`, gedeckelt bei
  ×100.000 - ganzzahlig und in Python, JavaScript und Rust bitgleich
  (`lamamines_logic.py`). Er liegt nie unter ×1,01 und steigt mit jedem
  Edelstein. Weil C(N−k, M) / C(N, M) genau die Chance ist, k Edelsteine in
  Folge zu finden, bringt **jede Auszahl-Strategie 96,2 bis 97 % zurück** -
  egal, wann man aussteigt. Nur drei Stufen mit 1 Mine auf 6x6 und 7x7 hebt die
  Untergrenze an (höchstens 98,9 %), und wer bis an den Deckel spielt, bekommt
  im Schnitt weniger zurück. Beispiel 5x5 mit 3 Minen: ×1,10, ×1,25, ×1,44 …
  bis ×2.231 für alle 22 Edelsteine. Wer ×100.000 erreicht, bekommt den
  **Höchstgewinn** sofort ausgezahlt.
- **Rest-Übertrag**: Was beim Abrunden einer Auszahlung an Hundertsteln übrig
  bleibt, wird zur nächsten gutgeschrieben - auch kleine Einsätze verlieren
  nichts.
- **Anzeige**: aktueller und nächster Multiplikator, die Chance, dass das
  nächste Feld sicher ist (`(N−M−k) / (N−k)` in %), der Gewinn beim nächsten
  Feld und eine Leiste mit den nächsten 5 Stufen. Edelsteine in vier Farben;
  das **Maskottchen-Lama mit Grubenhelm und Stirnlampe** füllt seine Lore,
  freut sich über jeden Edelstein und steht nach einer Mine voller Ruß da.
- **Mine getroffen**: Explosion und Kamera-Wackeln, danach wird das **ganze
  Feld abgedunkelt aufgedeckt** - so sieht man, wo die übrigen Minen lagen;
  nach dem Auszahlen ebenso. Münzregen ab ×3, Konfetti ab ×10, beim Abräumen
  und beim Höchstgewinn. Oben rechts steht der Verlauf der letzten 12 Runden.
- **Auto-Modus** (**A**): Felder im Auto-Fenster auf dem Raster markieren
  (Klick oder Pfeile + Leertaste, **Entf**/**Rücktaste** löscht; gemerkt je
  Feldgröße), Rundenzahl wählen (5/10/25/50/100) und auf Wunsch **Stopp bei
  Gewinn ab** bzw. **Stopp bei Verlust ab** (aus/100/500/1000/5000/10000/50000);
  **Enter** startet. Jede Runde deckt genau die markierten Felder schnell
  nacheinander auf und zahlt aus, wenn alle sicher waren. Auto läuft, bis die
  Rundenzahl erreicht ist, eine Grenze greift, die Chips nicht mehr reichen
  oder **STOPP**/Esc/Leertaste/Enter bzw. **A** gedrückt wird - eine laufende
  Runde bleibt dann offen und lässt sich von Hand zu Ende spielen. **Turbo**
  (**T**) setzt alle Zeiten auf 40 %.
- **Fairness-Check** (**F**): Vor jeder Runde steht der **Prüfcode** fest - der
  **SHA-256** eines geheimen Server-Seeds (32 Hex-Zeichen). Das Minenfeld
  entsteht aus Server-Seed, eigenem **Client-Seed** (8 Zeichen, per Knopf
  **Neuer Client-Seed** neu gewürfelt - nur außerhalb einer Runde) und
  Rundennummer: Zufall per `seedrand` (mulberry32), die Felder 0 … N−1 werden
  gemischt, die ersten M sind Minen. Nach der Runde wird der Seed gezeigt, der
  Prüfcode der nächsten Runde steht da schon fest. Das Fenster listet die
  letzten 12 Runden (Nr., Prüfcode, Server- und Client-Seed; Mausrad blättert,
  Klick wählt eine Zeile); **Nachrechnen** prüft den Prüfcode und berechnet das
  Minenfeld neu - ✓ oder ✗ samt Mini-Vorschau des Feldes.
- **Statistik** (**S**): Kacheln für Runden, Gewonnen, Verloren, Eingesetzt,
  Ausgezahlt, Bilanz, größten Gewinn, besten Multiplikator, **deine Quote**,
  gefundene Edelsteine, getroffene Minen und abgeräumte Felder, dazu eine
  **Bilanzkurve** der letzten 100 Runden.
- **Lama-Bank**: dasselbe Konto wie Blackjack, Poker, Casino und Lama Crossing;
  SPIELEN bucht den Einsatz und speichert die neue Runde in einem Zug
  (`lamabank.bet_and_pay`). Der Highscore ist der Höchststand von 1000 +
  Lama-Mines-Bilanz, unter 10 Chips gibt es einen **Bank-Kredit** auf 1000.
  **Fortsetzen**: Eine laufende Runde wird bei jedem aufgedeckten Feld
  gespeichert (Seeds, Rundennummer, Feldgröße, Minen, Einsatz und die
  aufgedeckten Felder); beim nächsten Start wird das Feld aus dem Seed neu
  berechnet und es geht genau dort weiter.
- Bedienung: vor der Runde **Leertaste**/**Enter** = SPIELEN, **1-4** =
  Feldgröße, **←**/**→** = Minen, **↑**/**↓**/**+**/**-**/Mausrad = Einsatz; in
  der Runde Klick oder **Pfeiltasten** + **Leertaste** = Feld aufdecken (die
  erste Leertaste zeigt den Cursor), **R** = Zufallsfeld, **Enter**/**C** =
  auszahlen; nach einer Runde startet die Leertaste gleich die nächste. **A** =
  Auto, **S** = Statistik, **F** = Fairness, **T** = Turbo, Esc schließt ein
  offenes Fenster bzw. stoppt Auto. Die Tasten sind fest und hängen nicht an
  der Steuerungs-Belegung in den Optionen. Laufende Runde, Verlauf,
  Fairness-Log, Statistik, Rest-Übertrag und Auto-Markierungen liegen je Profil
  im Abschnitt `casino` der `mem.json`; Feldgröße, Minen, Einsatz, Turbo und
  die Auto-Einstellungen im Abschnitt `lamamines` der `settings.json`. Erfolge
  **Schatzkammer** (ganzes Feld abgeräumt), **Tief gegraben** (Auszahlung ab
  ×10) und **Diamantenlama** (Auszahlung ab ×1000) sowie ein Meilenstein von
  3000 und Meister 10000.

**Lama Plinko**
- **Plinko mit Lama-Chips**: Einsatz je Kugel wählen (1 bis 1000), **WERFEN**
  drücken - ein **goldenes Wollknäuel** springt durch das Stiftbrett und landet
  in einem Fach mit **Multiplikator**: wenig in der Mitte, viel am Rand. Das
  Lama oben am Brett nickt bei jedem Abwurf.
- **8 bis 16 Reihen** (**←**/**→**, Standard 12) und **3 Risikostufen**
  (**1-3**: Niedrig, Mittel, Hoch). Das äußerste Fach zahlt bei 16 Reihen ×16
  (Niedrig), ×110 (Mittel) oder ×1000 (Hoch), bei 8 Reihen ×5,3, ×13 oder ×32;
  die Mitte ×0,7, ×0,4 oder ×0,2. Reihen und Risiko sind gesperrt, solange
  noch Kugeln fliegen oder Auto läuft.
- **27 feste Tabellen** (9 Reihenzahlen × 3 Risiken) in `lamaplinko_logic.py`,
  einmal erzeugt mit `devtools/gen_plinko_tables.py`: symmetrisch, nach außen
  nie fallend, „schöne" Zahlen und jede mit einer **exakt berechneten Quote
  zwischen 96,9 und 97,05 %**. Ein **Rest-Übertrag** der Hundertstel sorgt
  dafür, dass auch Einsatz 1 bei rund 97 % bleibt.
- **Ergebnis**: n Münzwürfe aus dem Fairness-Zufall (Zufallszahl < 0,5 =
  rechts), das Fach ist die Zahl der Rechts-Würfe - genau die
  Binomialverteilung. Die Animation folgt diesem Pfad Stift für Stift:
  Bogensprünge, der getroffene Stift blinkt, ein kleines Zittern (nur Optik)
  macht es lebendig; eine Kugel fliegt 1,62 s (8 Reihen) bis 2,30 s (16
  Reihen). Das Fach wippt beim Treffer, ein Popup zeigt den Multiplikator, der
  Ton hängt vom Wert ab; Münzregen ab ×10, ab ×100 ein Gewinn-Banner mit
  Konfetti.
- **Viele Kugeln gleichzeitig**: höchstens **50** auf dem Brett. **Leertaste
  gedrückt halten** = Dauerfeuer im Takt Flugzeit/30 - so sind rund 30 Kugeln
  zugleich unterwegs; **Turbo** (**T**) halbiert Flugzeit und Takt. Der Einsatz
  lässt sich auch im Flug ändern. Die Maus über einem Fach zeigt
  Multiplikator, Wahrscheinlichkeit und den Gewinn beim aktuellen Einsatz;
  daneben zählt ein Kasten die Kugeln, die Bilanz und den besten Treffer der
  Sitzung.
- **Auto-Modus** (**A**): 10/25/50/100/250/500/1000 Kugeln, auf Wunsch mit
  Stopp bei Gewinn ab bzw. Verlust ab (aus/100/500/1000/5000/10000/50000); eine
  Kugel alle 0,24 s (Turbo 0,12 s) mit festem Einsatz. Auto läuft, bis die Zahl
  erreicht ist, eine Grenze greift, die Chips nicht reichen oder **STOPP**/Esc
  gedrückt wird. Im Fenster: **↑**/**↓** = Zeile, **←**/**→** = Wert,
  **Enter**/**Leertaste** = START bzw. STOPP.
- **Fairness-Check** (**F**) wie bei Lama Mines, aber **je Kugel**: Prüfcode
  vorab, Seed nach der Landung, das Fenster zeigt die letzten 12 Kugeln
  (**↑**/**↓**/Mausrad wählt eine aus). **Enter** bzw. **Nachrechnen** prüft
  den Prüfcode, würfelt den Pfad neu aus und zeigt ✓ oder ✗ samt Mini-Vorschau
  des Wegs; **C** = neuer Client-Seed (nicht, solange Kugeln fliegen).
- **Statistik** (**S**): dieselben Kacheln und die Bilanzkurve wie bei Lama
  Mines (jede Kugel zählt als Runde), dazu die **Treffer-Verteilung** je
  Reihenzahl (**←**/**→** wechselt) als Balken gegen die erwartete
  Binomialkurve.
- **Lama-Bank**: dasselbe Konto wie Blackjack, Poker, Casino und Lama Crossing.
  Weil das Ergebnis beim Abwurf schon feststeht, wird sofort gebucht - Einsatz
  und Gewinn mit **einem** Speichern (`lamabank.bet_and_pay`), samt Verlauf,
  Fairness-Log, Statistik und Rest-Übertrag; Chips, Verlauf und Statistik
  zeigen den Gewinn erst bei der Landung. **Wollknäuel-Regen** fällt schon beim
  Abwurf, **Randtreffer** und **Plinko-Legende** bei der Landung - beim
  Verlassen werden sie für noch fliegende Kugeln nachgeholt, so geht nichts
  verloren. Highscore = Höchststand von 1000 + Lama-Plinko-Bilanz; ist das
  Konto leer und alle Kugeln sind gelandet, gibt es einen **Bank-Kredit** auf
  1000.
- Bedienung: Maus für alles, **Leertaste**/**Enter**/Klick auf WERFEN oder aufs
  Brett = werfen (halten = Dauerfeuer), **↑**/**↓**/**+**/**-**/Mausrad =
  Einsatz, **←**/**→** = Reihen, **1-3** = Risiko; **A** = Auto, **S** =
  Statistik, **F** = Fairness, **T** = Turbo, Esc schließt ein offenes Fenster
  bzw. stoppt Auto. Die Tasten sind fest und hängen nicht an der
  Steuerungs-Belegung. Verlauf, Fairness-Log, Statistik (samt Trefferzählern)
  und Rest-Übertrag liegen je Profil im Abschnitt `casino` der `mem.json`;
  Reihen, Risiko, Einsatz, Turbo und die Auto-Einstellungen im Abschnitt
  `lamaplinko` der `settings.json`. Erfolge **Randtreffer** (äußerstes Fach ab
  12 Reihen), **Wollknäuel-Regen** (25 Kugeln gleichzeitig unterwegs) und
  **Plinko-Legende** (äußerstes Fach bei 16 Reihen und Risiko Hoch) sowie ein
  Meilenstein von 3000 und Meister 10000.

**Lama Crash**
- **Crash-Spiel mit Lama-Chips**: Einsatz wählen (10 bis 10000), **SPIELEN**
  drücken - nach einem kurzen 3-2-1 steigt das Lama im **Ballon** auf, und der
  **Multiplikator** wächst mit der Zeit: ×2 nach 11,6 s, ×10 nach 38,4 s, ×100
  nach 76,8 s und ×100.000 nach 192 s (3,2 Minuten). Ab ×1,01 darf man
  jederzeit **AUSSTEIGEN** - das Lama springt mit dem Fallschirm ab, der Gewinn
  (Einsatz × Multiplikator) wird sofort gutgeschrieben; platzt der Ballon
  vorher, ist der Einsatz weg. Nach dem Aussteigen fliegt der leere Ballon
  weiter, bis er platzt („Alle draußen – Ballon steigt"), so sieht man, wie weit
  es gegangen wäre; **Vorspulen** (Leertaste) läuft 16-mal so schnell.
- **Zwei Einsätze** in derselben Runde (ein zweites Lama im Korb, per Schalter
  auf Karte 2 oder **Z**), jeder mit eigenem Betrag und eigenem **Auto-Ziel**
  (×1,01 bis ×100.000, **←**/**→**, **X** an/aus): Erreicht der Ballon das
  Ziel, wird genau beim Ziel ausgezahlt - auch wenn er im selben Moment platzt.
  Jeder Einsatz lässt sich einmal **halbieren** (**H** bzw. HÄLFTE für alle,
  **Q**/**W** oder der Knopf auf der Karte einzeln): Die Hälfte fliegt als
  Geldsack am Mini-Fallschirm davon, der Rest steigt weiter.
- **Platzpunkt** aus dem Fairness-Zufall: Zufallszahl r (32 Bit), dann
  `⌊97 · 2³² / (2³² − r)⌋` in Hundertsteln, mindestens ×1,00 und höchstens
  ×100.000 - ganzzahlig und in Python, JavaScript und Rust bitgleich
  (`lamacrash_logic.py`). Die Chance, ×x zu erreichen, ist damit 97 % / x (auf
  ganze 2⁻³² abgerundet): **jedes Ausstiegsziel bringt 96,997 bis 97,000 %
  zurück**, egal ob ×1,01 oder ×100.000; rund 3,96 % der Runden platzen schon
  bei ×1,00, bevor man aussteigen kann. Die Steigkurve ist ebenfalls
  ganzzahlig (`mu += mu // 1000` je Takt von 1/60 s), und der Ballon platzt
  erst, wenn der Multiplikator den Platzpunkt **überschreitet** - wer von Hand
  bei ×m aussteigt, gewinnt also unter genau derselben Bedingung wie ein
  Auto-Ziel ×m. Wer ×100.000 erreicht, landet auf dem **Mond**: Höchstgewinn
  für alle noch fliegenden Einsätze.
- **Höhenwelten**: Lamawiese (Weide, Zaun, Lama-Stall) → Wolkenmeer (ab ×1,5,
  Vogelschwärme) → Gewitterfront (ab ×3, Regen, Blitze, Flugzeuge) → Nordlicht
  (ab ×10, tanzende Polarlichter) → Weltall (ab ×100, Erdkrümmung, Satelliten,
  Sternschnuppen, ferne Planeten - und der Mond wird immer größer) →
  **Mondlandung** mit Fahne, Konfetti und Münzregen. Jede neue Welt wird beim
  Erreichen mit ihrem Namen angesagt, die Welten blenden weich ineinander
  über. Darüber läuft die Multiplikator-Kurve mit mitwachsenden Achsen, und
  der Ballon bläht sich mit der Höhe immer weiter auf.
- **Spannungsmusik** in sechs Stufen, je Höhenwelt eine (`lamacrash_music.py`):
  Von der Wiese bis ins Weltall wird das Stück mit jeder Welt einen Ganzton
  höher und schneller (96 bis 150 bpm), auf dem Mond erklingt eine Fanfare;
  **M** schaltet die Musik an oder aus. Ab ×2 quietscht der gedehnte Ballon.
  Nichts davon hängt vom Platzpunkt ab - Bild und Ton verraten nichts (ein
  Zwillingstest im Audit vergleicht zwei Runden mit Platzpunkt ×5 bzw. ×50 bis
  ×4,99 Bild für Bild und Ton für Ton).
- **Platzen**: Knall, Gummifetzen und Kamera-Wackeln; Lamas, die noch im Korb
  sitzen, purzeln samt Korb aus dem Bild. Beim Aussteigen gibt es Münzregen ab
  ×3, Konfetti ab ×10 und ein Gewinn-Banner ab ×100. Oben rechts stehen die
  Platzpunkte der letzten 12 Runden als farbige Pillen (rot unter ×2, grün ab
  ×2, gold ab ×10, violett ab ×100).
- **Auto-Modus** (**A**, nur zwischen den Runden): 5/10/25/50/100/250 Runden,
  auf Wunsch mit **Stopp bei Gewinn ab** bzw. **Stopp bei Verlust ab**
  (aus/100/500/1000/5000/10000/50000) und Turbo. Jeder aktive Einsatz bekommt
  dabei ein Auto-Ziel, im Leerflug wird automatisch vorgespult, von Hand
  aussteigen geht weiterhin. Auto läuft, bis die Rundenzahl erreicht ist, eine
  Grenze greift, die Chips nicht reichen oder **STOPP**/Esc/**A** gedrückt
  wird - eine Runde im Flug fliegt dann normal zu Ende. Im Fenster: **↑**/**↓**
  = Zeile, **←**/**→** = Wert, **Enter**/**Leertaste** = START bzw. STOPP.
  **Turbo** (**T**) lässt den Ballon dreimal so schnell steigen und kürzt
  Countdown und Pausen.
- **Fairness-Check** (**F**) wie bei Lama Mines: Vor jeder Runde steht der
  **Prüfcode** fest (SHA-256 des geheimen Server-Seeds), der Platzpunkt
  entsteht aus Server-Seed, eigenem **Client-Seed** und Rundennummer
  (`fairness.round_rand`, mulberry32), nach der Runde wird der Seed gezeigt.
  Das Fenster listet die letzten 12 Runden (**↑**/**↓** wählt, **C** = neuer
  Client-Seed, solange keine Runde läuft); **Nachrechnen** (**Enter**) prüft
  den Prüfcode, berechnet den Platzpunkt neu und kontrolliert die Ausstiege -
  ✓ oder ✗ samt Mini-Kurve.
- **Statistik** (**S**): Kacheln für Runden, Gewonnen, Verloren, Deine Quote,
  Eingesetzt, Ausgezahlt, Bilanz, Größter Gewinn, Bester Faktor,
  **Max. Platzpunkt**, Halbierungen und Haarscharf, dazu die **Bilanzkurve**
  der letzten 100 Runden und die **Verteilung der Platzpunkte** in sieben
  Fächern (×1 bis ×1000+) als Balken gegen die erwartete Verteilung (goldene
  Punkte).
- **Lama-Bank**: dasselbe Konto wie Blackjack, Poker, Casino, Lama Crossing,
  Lama Mines und Lama Plinko. SPIELEN bucht den Gesamteinsatz und speichert
  die neue Runde in einem Zug (`lamabank.bet_and_pay`), jeder Ausstieg schreibt
  den Gewinn sofort gut; ein **Rest-Übertrag** der Hundertstel sorgt dafür,
  dass auch kleine Einsätze nichts verlieren. **Fortsetzen**: Wer mitten im
  Flug geht, macht beim nächsten Start nach einem Countdown am selben Takt
  weiter - der Platzpunkt steht ja fest; wer erst im Leerflug geht, dessen
  Runde wird sofort abgeschlossen. Verliert das Fenster den Fokus, während
  Geld fliegt, **pausiert** das Spiel von selbst (neuer Engine-Haken
  `pause_on_blur` in `game_base.py`, ausgewertet von `main.py`). Highscore =
  Höchststand von 1000 + Lama-Crash-Bilanz; unter 10 Chips gibt es einen
  **Bank-Kredit** auf 1000.
- Bedienung: vor der Runde **Leertaste**/**Enter** = SPIELEN,
  **1**/**2**/**Tab** = Karte wählen, **↑**/**↓**/**+**/**-**/Mausrad =
  Einsatz, **←**/**→** bzw. Mausrad über dem Ziel = Auto-Ziel, **X** = Ziel
  an/aus, **Z** = zweiter Einsatz; im Flug **Leertaste**/**Enter**/**C** = alle
  aussteigen, **1**/**2** = einzeln, **H** = Hälfte, **Q**/**W** = Hälfte
  einzeln; danach **Leertaste** = Vorspulen. **A** = Auto, **S** = Statistik,
  **F** = Fairness, **T** = Turbo, **M** = Musik, Esc schließt ein Fenster bzw.
  stoppt Auto (sonst Pause). Die Fenster lassen sich nur öffnen, solange kein
  Einsatz fliegt; die Tasten sind fest und hängen nicht an der
  Steuerungs-Belegung. Runde, Verlauf, Fairness-Log, Statistik und
  Rest-Übertrag liegen je Profil im Abschnitt `casino` der `mem.json`;
  Einsätze, Ziele, zweiter Einsatz, Turbo, Musik und die Auto-Einstellungen im
  Abschnitt `lamacrash` der `settings.json`. Erfolge **Haarscharf** (von Hand
  ab ×2 höchstens 5 % vor dem Platzen ausgestiegen), **Polarlicht-Pilot**
  (Ausstieg ab ×10) und **Weltraumlama** (Ausstieg ab ×1000) sowie ein
  Meilenstein von 3000 und Meister 10000.

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
  den bisherigen Rekord als Chip an. Bei vielen Modi und kleiner Auflösung rückt
  er **kompakt** zusammen: Optionen, Matches, Wiki und Zurück stehen dann in einer Reihe,
  die Schrift passt sich an - nichts läuft mehr aus dem Bild.
- **Einheitlicher Spiel-Look**: alle 50 Spiele nutzen dieselbe Theme-Palette und
  -Schrift wie das Menü - HUDs, Setup-Screens und Overlays folgen dem in den
  Optionen gewählten Design (v4.1 / v4 / Classic), die Spielfelder behalten ihre
  Identitätsfarben. Auflösungswechsel mitten im Spiel übernimmt jedes Spiel
  sauber, und die Spielnamen im Menü sind sprachabhängig
  (z. B. „Schach" → "Chess" / « Échecs »).
- **In-Game-Wiki** („LamaWiki"): ausführliche Hilfe zu jedem Spiel (Steuerung,
  Modi, Punkte, Tipps) plus allgemeine Seiten - mit **Suchfeld**, Kategorien,
  scrollbaren Artikeln und Tastenkappen-Chips, in allen 14 Sprachen.
  Erreichbar über den Sidebar-Button **„Wiki / Hilfe"** und aus dem
  Vorspiel-Screen jedes Spiels (öffnet direkt dessen Seite).
- **Erfolge, Seltenheit & Level**: **274 Erfolge** in fünf Kategorien - 60
  sammlungsweite Ziele (Partien, Spielzeit, Rekorde, Siege, Spieltage,
  Tagesserien, Duelle …), 41 **Punkte-Meilensteine**, 34 **Meister-Stufen**
  (die zweite, deutlich höhere Punktmarke), für jedes der 50 Spiele einen
  **Stammgast**-Erfolg (25 Partien) und 89 **besondere Momente** (Schachmatt,
  KNIFFEL, die 4096er-Kachel, Hole-in-One, Hattrick im Air Hockey, vier Geister
  mit einer Kraftpille …). Jeder Erfolg hat eine **Seltenheitsstufe**
  (gewöhnlich 10 bis mythisch 300 Punkte); die Punkte ergeben dein **Level**
  mit 14 **Rängen** vom Lama-Küken bis zum Sternenlama. Freischalten zeigt eine
  Einblendung in der Farbe der Stufe, ein Aufstieg eine eigene mit Fanfare.
- **Statistik mit Diagrammen**: fünf Reiter - Erfolge (mit Filter, Punkten und
  XP-Balken), Statistiken (Karten, Top-10-Balken, Sieg/Niederlage-Ring,
  Tabelle), **Kalender** (Spieltage-Karte, Serien, Wochentage, Tageszeit),
  **Verlauf** (alles mit Zeitstempel, dazu der Punkteverlauf je Spiel als
  Linie) und **Profile** (Geräte-Bestenliste, Duell-Bilanz). Obendrauf der
  **PyGameZ-Rückblick** mit stärkstem Tag, Lieblingszeit und längster Sitzung.
  Alles liegt in der Datei **`stats.pgzlamastats`** und lässt sich exportieren,
  weitergeben und wieder einlesen - auch zwischen Desktop und Web-Version.
- **Spielerprofile**: mehrere Spieler an einem Gerät, jeder mit eigenen
  Highscores, Erfolgen, Level, Statistiken, Chips und Spielständen (bis zu
  acht). Der Knopf **„Profile"** legt an, benennt um und wechselt; im
  Mehrspieler wählst du vor der Partie das Profil für Spieler 2 (oder spielst
  als Gast), im Spiel stehen dann die Namen statt „Spieler 1/2", und jede
  Partie landet in der **Duell-Bilanz**.
- **Replays**: **sechs Spiele** zeichnen ihre Runden auf - **Minigolf**,
  **Bowling**, **Billard**, **Pinball**, **Snake** und **Tetris**. Gespeichert
  wird die tatsächliche Bewegung, Bild für Bild (30 Samples je Sekunde), nicht
  die Eingabe; die Wiederholung sieht deshalb exakt aus wie die gespielte Runde,
  samt HUD und Scorekarte. Bei Minigolf, Bowling, Billard und Pinball ist eine
  Sequenz ein Schlag, Wurf, Stoß bzw. Ball; Snake und Tetris laufen durch und
  werden in **Kapitel** von zehn Sekunden geschnitten (Schlüsselbild +
  Änderungen, damit das Spulen flott bleibt). Am Rundenende zeigt **P** die
  Wiederholung, **S** legt sie ins Archiv; der Sidebar-Button **„Replays"**
  öffnet es jederzeit wieder (ein Reiter je Spiel, Pause, Sprung zur nächsten
  Sequenz, Tempo 0,5x bis 4x, klickbare Fortschrittsleiste, Löschen per Entf).
  Je Spiel passen 20 Aufnahmen ins Archiv; abschaltbar beim ersten Start und in
  den Optionen.
- **Replays teilen**: **E** schreibt eine Aufnahme als **`.lamapgzreplay`**-Datei
  (im Archiv wie direkt nach der Runde), **I** liest so eine Datei wieder ein -
  sie landet im Reiter des passenden Spiels und trägt dort einen kleinen Pfeil.
  Eine Datei enthält genau ein Replay und wandert wie eine Minigolf-Bahn oder
  ein Geometry-Dash-Level von Rechner zu Rechner; schon vorhandene Aufnahmen
  werden nicht doppelt eingelesen. Die **Browser-Version** (`web/`) kann alles
  davon ebenfalls - und die Dateien sind **austauschbar**: eine am PC
  aufgenommene Runde läuft im Browser und umgekehrt. Dort liegt das Archiv im
  Browser-Speicher, deshalb passen 5 Aufnahmen je Spiel hinein.
- **Match-Historie**: jedes der 50 Spiele merkt sich seine Matches. Der Knopf **„Matches"** im
  Vorspiel-Screen öffnet **Letzte Matches** (Verlaufskurve der letzten 30 Ergebnisse, Rekorde golden,
  dazu die letzten 10 Matches) und **Alle Matches** (Filter nach Modus **M**, Sortierung neueste/beste
  **O**). Gespeichert werden Datum, Modus, Punkte bzw. Chip-Gewinn, Sieg/Niederlage/Remis, Dauer und
  Details wie Einsatz, Multiplikator, Münzen oder Level - je Spiel die letzten **250** Matches (die
  Summen laufen unbegrenzt weiter) in **`matches.pgzlamamatches`**, getrennt je Profil. Gleich in der
  Web-Version, der Rust-Fassung und auf pygamez.com.
- **Tages-Archiv**: Tagesstrecke (Crossy Road), Tagesrätsel (Sudoku) und Tageswort (Wordle) lassen
  sich für die **letzten 30 Tage** und jeden schon gespielten Tag nachspielen - mit demselben Seed wie
  damals und eigenem Bestwert bzw. eigener Bestzeit je Tag. Im Setup wechseln die Pfeile neben dem
  Datum oder **Bild auf/ab** den Tag, **K** öffnet den Kalender. Archiv-Tage ändern keine Serien und
  keine Tagesstatistik; gespeichert wird kompakt (eine Tageszahl statt eines Datums - ein Jahr
  Crossy-Archiv sind rund 4 KB).

### Bedienung

- Spiel links im Menü per Button wählen. Danach erscheint ein **Vorspiel-Screen**:
  **Einzelspieler** oder **Mehrspieler** wählen, zu den **Optionen** gehen oder
  zurück. Pfeile/Maus zum Wählen, Enter startet.
- **ESC** = Pause / weiter (in Menüs: zurück). Spiele, die sonst ohne dich
  weiterliefen, pausieren beim Fokusverlust von selbst (Engine-Haken
  `pause_on_blur` - bisher Lama Crash, solange ein Einsatz fliegt).
- **F11** (oder Button „Vollbild an/aus") = Vollbild ein/aus. Das Pygame-Display
  bleibt eingebettet und wird seitenverhältnistreu hochskaliert (schwarze Ränder
  bei abweichendem Seitenverhältnis). Das Fenster lässt sich frei skalieren.
- **„Zurück zum Menü"** beendet das Spiel und speichert den Highscore - ebenso
  der Wechsel in ein anderes Spiel über die Sidebar.
- **Feste Zusatztasten**: Einige Spiele haben neben den fünf belegbaren Aktionen
  eigene Tasten (z. B. Halten **C** und Links-Drehen **Z** in Tetris, Rückgängig
  **U** in 2048, Schach und Sudoku). Sie greifen nur, wenn die Taste in den
  Optionen keiner Aktion zugeordnet ist, und stehen im Setup-Hinweis sowie im
  Wiki. Gehaltene Tasten werden sauber erkannt und bei Pause oder Alt-Tab
  losgelassen - nichts „klemmt".
- **„Beenden"** schließt Pygame und Tkinter sauber.

### Optionen, Steuerung & Sound

Über den Button **„Optionen / Steuerung"** (links) oder aus dem Vorspiel-Screen
öffnet sich der Options-Bildschirm. Er ist in **drei Reiter** gegliedert
(**Allgemein / Steuerung / Erscheinungsbild**; wechseln per Klick oder
Tab-Taste):

- **Allgemein**: **Sound** an/aus, **Lautstärke** und **Haptik**
  (Gamepad-Vibration, nur mit angeschlossenem Controller wirksam) sowie
  **Auto-Auflösung**, **Auflösung**, **FPS** und **Sprache** – jeweils per
  Links/Rechts umschalten. Unter **Aufnahme** steht **Replays aufzeichnen**:
  aus bedeutet, dass Minigolf und Bowling nichts mitschneiden.
- **Steuerung**: **Vorlagen** (*WASD + Pfeile*, *WASD + IJKL*, *Pfeile + WASD*)
  und **jede einzelne Taste** für Spieler 1 und Spieler 2 frei belegen:
  Zeile wählen, Enter drücken, gewünschte Taste drücken (Esc bricht ab).
- **Erscheinungsbild**: das **UI-Design** wählen – **UI v4.2** (Standard:
  Midnight Glass – tiefer Mitternachts-Verlauf mit langsam treibenden Lichtern
  in Indigo, Türkis und Magenta, feinem Filmkorn, spärlichen Sternen und
  Panels wie mattes Milchglas mit heller Lichtkante), **UI v4.1** (wie UI v4,
  aber lebendiger – dezente Sterne sowie Saturn und ein Schwarzes Loch im
  Hintergrund des Startbildschirms), **UI v4.1.1** (wie v4.1, statt
  Sternenhimmel aber ein gekacheltes **Zickzack-Muster** in Schwarz und
  Anthrazit), **UI v4.1.2** (dasselbe Muster in den Blautönen der Palette –
  Akzentblau als dominante Farbe, dunkleres Blau als Grundfläche),
  **UI v4.1.3** (dasselbe Muster im Indigo-Akzent von UI v4 auf Schwarz),
  **UI v4.1.4** (im Graphit-Ton von UI v4 auf Schwarz), **UI v4** (komplett
  ruhiges, flaches Graphit-Design mit einem Indigo-Akzent), **UI v3** (die
  bisherige klassische UI mit Sternenhimmel, Aurora-Lichtern und
  Glow-Effekten), **UI v2** (die allererste überarbeitete UI: Navy-Verlauf mit
  Sternenfeld, Buttons mit statischem Glow und Pfeil, Titel mit Schatten –
  ganz ohne Animationen) oder **UI v1** (der Look vor dem UI Rework:
  einfarbiger dunkler Hintergrund, flache Buttons, keine Effekte). Alle Karten
  zeigen eine kleine Vorschau; die Wahl wirkt sofort auf die komplette
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
start.command        Startskript für macOS (Doppelklick im Finder, ruft start.sh auf)
pyinstall-dev/
  pyinstall.bat      EXE-Build (Windows): packt alles in eine builds\PyGameZ.exe
main.py              Tkinter-Oberfläche, Pygame-Einbettung, zentrale Game-Loop
game_base.py         Game-Basisklasse (update/draw/handle_event) + InputEvent + Helfer
settings.py          Einstellungen (Sound/Haptik/Tastenbelegung/Spiel-Optionen mit Prüfregeln) laden/speichern (JSON)
audio.py             Prozedurale Soundeffekte, Musik-Schleifen + Gamepad-Rumble
menu.py              Sprach-, Vorspiel- (Modus) und Options-Screen (Sound/Steuerung)
ui.py                Gemeinsames UI-Toolkit für alle Pygame-Screens: zehn Designs, Hintergrund, Titel, Buttons, Panels, Effekte
logo.py              Zentrale Logo-Verwaltung: welches Logo aus logo/ geladen wird (Fenster-Icon, Menü)
highscore.py         Laden/Speichern der Highscores (Abschnitt in mem.json)
store.py             Zentrale Speicherdatei mem.json (gerätweit: mem; je Profil: highscores, achievements + Spielstände), atomar mit .bak-Sicherung
profiles.py          Spielerprofile: anlegen, wechseln, umbenennen, Namensprüfung, Farbe/Symbol
stats.py             Statistik in stats.pgzlamastats: Summen, Spieltage, Tageszeit, Punkteverläufe, Verlauf, Duelle, Export/Import
achievements.py      Erfolge: Definitionen, Seltenheitsstufen, Freischalt-Logik, Toast-Einblendung
levels.py            Levelsystem: Erfolgspunkte -> Level und 14 Ränge
charts.py            Diagramme für die Statistik-Screens (Balken, Linien, Kalender-Heatmap, Ringe, Tooltip)
progress.py          Erfolge-&-Statistik-Screen (fünf Reiter, scrollbar)
profileview.py       Profil-Screen, Spieler-2-Auswahl und die Startlevel-Karte
replay.py            Aufzeichnung, Archiv & Teilen der Wiederholungen (replay.json, .lamapgzreplay)
replayview.py        Replay-Screen: Archiv-Liste, Wiedergabe, Teilen/Einlesen
matches.py           Match-Historie je Spiel und Profil (matches.pgzlamamatches): Einträge, Summen, Kappe 250
matchview.py         Matches-Screen: Letzte Matches (Kurve + letzte 10) und Alle Matches (Filter, Sortierung)
dailyarchive.py      Tages-Archiv: letzte 30 Tage + gespielte Tage, kompakte Speicherung, Kalender-Fenster
ugc.py               Eigene Inhalte (Minigolf-Bahnen, Geometry-Dash-Level): Speicher, Prüfung, Export/Import (ugc.json)
swear.py             Wortfilter für Namen und ids (lang/swear/*.yml, alle 14 Sprachen)
filepick.py          Datei-Dialoge ("Exportieren als ...", "Importieren") und Downloads-Ordner
prestige.py          Prestige-System für Snake
competitive.py       Kennzahlen für den Competitive-Modus von Snake (Level, Slot, Wett-Äpfel)
ngb.py               Visuelle Personalisierung ("Mods"): Kopffarbe + Koordinaten-Raster + Menü (Abschnitt ngb je Profil)
lamabank.py          Lama-Bank: gemeinsames Chip-Konto von Blackjack, Poker, Casino, Lama Crossing, Lama Mines, Lama Plinko und Lama Crash (Abschnitt casino in mem.json)
seedrand.py          Zufallsgenerator mit bitgleichen Zahlen in Python und Browser (Tagesmodi, neue Rätsel)
fairness.py          Fairness-Check von Lama Mines, Lama Plinko und Lama Crash: Server-/Client-Seed, Nonce, SHA-256-Prüfcode, Nachrechnen (Gegenstücke: web/js/games/fairness.js, rust/crates/pgz-core/src/fairness.rs)
chipstats.py         Statistik von Lama Mines, Lama Plinko und Lama Crash (Runden, Einsatz, Auszahlung, Quote, Bilanzkurve) für das Fenster S (dazu chipstats.js, chipstats.rs)
i18n.py              Übersetzungs-Engine (lädt lang/*.json, t("schlüssel"))
lang/
  de.json  en.json  fr.json  es.json  pt.json   Sprach-Strings (ein Platzhalter-Schlüssel je Text)
  lang.expansion/                           pl, tr, da, no, sv, fi, cs, sl, hr
  swear/                                    Wortfilter-Listen je Sprache (Regex, .yml)
lamawiki/
  lamawiki.py          In-Game-Wiki (Suche, Kategorien, Artikel-Renderer)
  de.json  en.json  fr.json  es.json  pt.json   Wiki-Inhalte (eine Seite je Spiel + Allgemeines)
  lang.expansion/                           pl, tr, da, no, sv, fi, cs, sl, hr
woordlistz/
  build_wordlists.py   Baut die Wordle-Wortlisten aus Wörterbüchern + Häufigkeitslisten neu
  de/ en/ fr/ ... hr/  answers.txt + allowed.txt (5 Buchstaben), answers4/6/7.txt + allowed4/6/7.txt (4, 6, 7 Buchstaben), 14 Sprachen
devtools/            Werkzeuge für Entwickler (werden nicht in die .exe gepackt)
  merge_staging.py           Spielt Übersetzungen und Wiki-Seiten aus devtools/staging/ in alle 14 Sprachdateien ein
  build_chess_puzzles.py     Baut die 200 Schachrätsel aus der Lichess-Rätseldatenbank (CC0)
  build_sudoku_killer.py     Erzeugt die 400 eindeutig lösbaren Killer-Sudokus
  build_crossyroad_models.py Schreibt die Voxel-Modelle von Crossy Road für die Web-Version
  build_geodash_levels.py    Baut die 16 Geometry-Dash-Kampagnenlevel und beweist per Solver, dass jedes samt Münzen schaffbar ist
  build_geodash_extra.py     Baut die Bibliothek (80 Level in 10 Kategorien) und die 12 Vorlagen, jedes vom Solver bewiesen
  build_geodash_solver.py    Solver mit dem echten Schritt-Code (Lösungen in geodash_proofs.json)
  geodash_sections.py        Sektions-Baukasten für build_geodash_extra.py: kurze Level-Stücke, die sich aneinanderreihen lassen
  gen_plinko_tables.py       Erzeugt die 27 Plinko-Tabellen (8-16 Reihen × 3 Risiken) mit exakt berechneter Quote für lamaplinko_logic.py
  export_chiplogic_ref.py    Schreibt die Prüfwerte der Chip-Regeln (Fairness, Statistik, Mines, Plinko, Crash) für die Rust-Unit-Tests
  export_achievements.py     Schreibt die Erfolgs- und Level-Definitionen für die Web-Version (web/js/core/achdefs.js)
games/
  snake.py  pong.py  airhockey.py  tictactoe.py  breakout.py
  tetris.py  tetris_core.py  tetris_ai.py
  invaders.py  asteroids.py  pacman.py  flappy.py  doodle.py
  game2048.py  minesweeper.py  sudoku.py  sudoku_gen.py  sudoku_draw.py
  frogger.py  memory.py  solitaire.py  cards.py  aimtrainer.py
  connect4.py  tanks.py  blackjack.py  tunnelracer.py
  labyrinth.py  maze_gen.py  reversi.py  kniffel.py  wordle.py  wordle_words.py
  trexrunner.py  dame.py  poker.py  muehle.py
  chess.py  chess_engine.py  chess_draw.py
  simon.py  billiard.py  slidepuzzle.py  mastermind.py
  bubbleshooter.py  hangman.py  hangman_words.py  blockjump.py
  lamatowerdefense.py  minigolf.py  minigolf_gen.py
  minigolf_draw.py  minigolf_edit.py  pinball.py  bowling.py
  crossyroad.py  crossyroad_world.py  crossyroad_draw.py
  geodash.py  geodash_core.py  geodash_draw.py  geodash_edit.py  geodash_extra.py  geodash_music.py
  battleship.py  battleship_core.py
  casino.py  casino_logic.py  casino_roulette.py  casino_slots.py  casino_draw.py
  lamacrossing.py  lamacrossing_logic.py  lamacrossing_draw.py
  lamamines.py  lamamines_logic.py  lamamines_draw.py
  lamaplinko.py  lamaplinko_logic.py  lamaplinko_draw.py
  lamacrash.py  lamacrash_logic.py  lamacrash_draw.py  lamacrash_world.py  lamacrash_music.py
  levels/              Level-Daten: snake-comp.json, chess-puzzles.json (+ Quellen-README), sudoku-killer.json, geodash.json, geodash_extra.json, geodash_templates.json
tests/
  arcade_casino_audit.py     Gesamt-Audit (Eingabe, seedrand Python = JS, Speichern, Sprachdateien, Vorspiel-Screens) + alle audit_*.py
  audit_tetris.py  audit_2048.py  audit_chess.py  audit_sudoku.py  audit_wordle.py
  audit_crossyroad.py  audit_geodash.py  audit_battleship.py  audit_casino.py  audit_lamacrossing.py   Headless-Audits je Spiel
  audit_lamamines.py  audit_lamaplinko.py  audit_lamacrash.py  audit_chiplogic.py   Lama Mines, Lama Plinko, Lama Crash und die gemeinsame Chip-Logik (Fairness, Statistik, Python = JS = Rust)
  newgames_audit.py  blockjump_audit.py  progress_audit.py  replay_audit.py
rust/                Die Rust-Fassung (eigenständige .exe, siehe rust/README.md)
  ENGINE.md            Portierungs-Vertrag: wie ein Spiel von Python nach Rust kommt
  crates/pgz-core/     Datenschicht ohne Grafik (store, settings, i18n, profiles,
                       highscore, levels, stats, achievements, seedrand, swear)
  crates/pgz-engine/   Fenster, Zeichnen, Eingabe, Ton, Game-Trait (ui.py + audio.py + game_base.py)
  crates/pgz-games/    Die portierten Spiele
  crates/pygamez/      Das Programm: Menü, Seitenleiste, Bildschirme
  tools/export_defs.py Erzeugt die Stammdaten (50 Spiele, 274 Erfolge, 10 Designs) aus den Python-Quellen
  tools/audit.py       Prüfstand: Übersetzungen, Spieleliste, Konventionen
rust.output/         Build-Ausgabe der Rust-Fassung (rust.output/release/pygamez.exe)
```

Die gewählte Sprache wird in `mem.json` gespeichert (im Abschnitt `mem`, neben
dem Abschnitt `highscores` in derselben Datei) und beim nächsten Start
automatisch geladen.

**Quellen & Lizenzen:** Die 200 Schachrätsel stammen aus der
[Lichess-Rätseldatenbank](https://database.lichess.org/#puzzles) (Lizenz
**CC0 1.0**, gemeinfrei - danke an lichess.org!); Details stehen in
`games/levels/chess-puzzles.README.md`. Die Quellen der Wordle-Wortlisten nennt
`woordlistz/README.md`.

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

#### macOS mit start.command

1. **Python installieren**, am besten **Python 3.13 von
   <https://www.python.org/downloads/macos/>** (der Installer bringt Tkinter mit).
   Mit Homebrew geht auch `brew install python python-tk` (ohne `python-tk` fehlt
   Tkinter). Apples vorinstalliertes `/usr/bin/python3` reicht nicht (altes Tk 8.5).
2. Projektordner holen (`git clone` oder ZIP) und **`start.command` doppelklicken**.
   Beim ersten Start legt das Skript `.venv` an und installiert pygame-ce (etwa eine
   Minute), danach startet die Sammlung.
3. Blockiert macOS das Öffnen („nicht verifizierter Entwickler"):
   **Systemeinstellungen → Datenschutz & Sicherheit → „Trotzdem öffnen"**.

Im Terminal geht dasselbe mit `./start.sh`. Meldet es „Permission denied" (kann nach
dem Entpacken einer ZIP passieren), hilft `bash start.sh` oder einmalig
`chmod +x start.sh start.command`.

#### Linux mit start.sh

```bash
sudo apt install python3 python3-venv python3-tk   # einmalig (Debian/Ubuntu)
./start.sh                                         # legt .venv an, installiert pygame, startet
```

Fedora: `sudo dnf install python3 python3-tkinter`, Arch: `sudo pacman -S python tk`.
Findet `start.sh` kein passendes Python, nennt es den passenden Befehl.

#### Andere Python-Version verwenden

`install-python.bat` richtet standardmäßig Python 3.13 ein. Wer 3.12 (oder eine
andere Version) bevorzugt, ändert in der Datei die Zeile `set "PYVER=3.13"` auf die
gewünschte Version und die winget-ID entsprechend (`Python.Python.3.12`).

#### Eigenständige EXE bauen (Windows)

```bat
pyinstall-dev\pyinstall.bat   :: baut builds\PyGameZ.exe (alles in einer Datei)
```

`pyinstall.bat` nutzt die `.venv` (und erstellt sie bei Bedarf), installiert
**PyInstaller** automatisch nach und packt das komplette Spiel - Python,
pygame, alle Spiele, Sprachen, Wiki und Logos - in **eine einzige
`PyGameZ.exe`** im Ordner **`builds\`**. Die Datei läuft auf jedem Windows-PC
ohne installiertes Python und lässt sich frei kopieren. Einstellungen und
Highscores (`settings.json`, `mem.json`) legt die .exe beim
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
**Pygame** is embedded as the game display inside the Tkinter window. Fifty
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

or run the [PyGameZ-2026-26-August.exe](https://github.com/herrlamatv/pygamez/releases/download/26.26.08-05.04/PyGameZ-2026-26-August.exe) (Latest Release)


#### macOS

Double-click **`start.command`** in Finder. All you need is Python 3.9+ with
Tkinter, easiest is **Python 3.13 from [python.org](https://www.python.org/downloads/macos/)**.

#### Linux / Git Bash

```bash
./start.sh            # sets up .venv + pygame on first launch, then starts
```

`start.bat` / `start.sh` / `start.command` automatically use the virtual
environment `.venv`. If it is missing, they create it and install pygame-ce from
`requirements.txt`. A detailed step-by-step guide is at the very bottom under
**[Installation Guide](#installation-guide)**.

### The games

| Game         | Modes           | Short description |
|--------------|-----------------|-------------------|
| **Snake**    | 1 / 2 players   | Deluxe Snake with 2D & 3D view, boost, 6 game modes (incl. Competitive), golden apples and prestige |
| **Pong**     | 1 / 2 players   | Classic vs. AI or player 2, switchable movement mode |
| **Air Hockey** | 1 / 2 players | 2D physics with momentum transfer, mouse control, AI and power-ups |
| **Tic-Tac-Toe** | 1 / 2 players | m,n,k game on 3x3 to 9x9, three AI strengths **or** local X vs. O |
| **Breakout** | 1 player        | Brick breaker with brick types, power-ups, combos and many levels |
| **Tetris**   | 1 / 2 players   | Modern Guideline rules (SRS, hold, 5-piece preview, T-Spins): Marathon, Sprint 40, Ultra 2:00, Versus vs. the AI (3 strengths) or 2-player versus with garbage lines |
| **Invaders** | 1 player        | Space Invaders: clear the waves, protect your lives |
| **Asteroids** | 1 / 2 players  | Inertia physics, waves, UFOs, power-ups, hyperspace - solo or co-op duel |
| **Pac-Man**  | 1 player        | Faithful clone: 4 ghost AIs, power pills, tunnel, fruit, levels |
| **Flappy Bird** | 1 player     | Gravity flight through pipes, coins, shield, day/night, medals |
| **Doodle Jump** | 1 player     | Auto-jump upward, platform types, springs, propeller, monsters |
| **2048**     | 1 player        | Number-sliding puzzle on 3x3 to 8x8: Classic, Time attack and Endless, undo, smooth animations, save & resume games |
| **Minesweeper** | 1 player     | The classic with safe first click, chording, smiley and best times |
| **Sudoku**      | 1 player     | 4 variants (Classic, X-Sudoku, Killer, Mini 6x6) with 400 levels each, Daily Sudoku, up to 3 stars per level, 4 assist modes, undo, saved progress |
| **Frogger**     | 1 player     | Road + river + 5 bays, bonus fly, crocodiles, time limit, 3 difficulties |
| **Memory**      | 1 / 2 players | Find pairs on 4x4 up to 8x6, flip animation, solo scoring or duel |
| **Solitaire**   | 1 player     | 5 variants (Klondike, Spider, FreeCell, Pyramid, TriPeaks) with drag & drop and undo |
| **Aim Trainer** | 1 player     | Chill 3D target shooting: mouse steers the camera, 4 modes (precision/reflex/moving/chill), 3 themes incl. a black hole |
| **Connect Four** | 1 / 2 players | The classic with falling-disc animation: 3 AI strengths (minimax) or a local duel |
| **Tank Duel**    | 1 / 2 players | 2D arena duel with ricochet shots, power-ups, 4 arenas, AI with 3 strengths |
| **Blackjack**    | 1 player     | Casino blackjack with a 4-deck shoe, double/split and 3:2 blackjack; plays with the llama chips of the shared Llama Bank |
| **Tunnel Racer** | 1 player     | 3D neon tube flight: endless mode + 30 levels, key or mouse steering, motion blur |
| **3D Maze**      | 1 player     | First-person raycaster (Wolfenstein style) with 50 seeded levels, orbs, minimap - or a 2D top-down view |
| **Reversi**      | 1 / 2 players | Othello on 8x8: trap and flip discs, 3 AI strengths (minimax) or a local duel |
| **Kniffel (Yahtzee)** | 1 / 2 players | Dice classic with 13 categories, upper bonus and Yahtzee; high-score chase or 2-player hotseat |
| **Wordle**       | 1 player     | Guess words with 4 to 7 letters: Endless, Daily word, Dordle and Quordle, hard mode, colour-blind palette, statistics with a bar chart, share your result, real word lists in 14 languages |
| **T-Rex Runner** | 1 player     | Endless desert run: variable jump, duck, cacti & pterodactyls, day/night cycle, rising speed, 3 difficulties |
| **Draughts (Dame)** | 1 / 2 players | 3 rule sets (German 8×8, International 10×10, Checkers), forced captures & flying kings, 3 AI strengths (minimax) or a local duel |
| **Poker**        | 1 player     | 3 selectable variants: Texas Hold'em vs AI, 5 Card Draw and Video Poker; betting rounds, blinds, llama chips of the shared Llama Bank |
| **Chess**        | 1 / 2 players | Full rules, Chess960 and chess clock, 6 AI strengths, 200 puzzles from the Lichess database, undo/hints, move list, PGN export or a local duel |
| **Nine Men's Morris** | 1 / 2 players | Placing/moving/flying phases, mills & captures, optional flying rule, 3 AI strengths or a local duel |
| **Simon**        | 1 / 2 players | Senso memory game: Classic/Speed/Reverse/Mixed modes + Duel, sound off/on/mixed, 4/6/9 pads, best per mode |
| **Billiards**    | 1 / 2 players | 8-ball, 9-ball & practice in 2D, fixed 3D view or free-orbit 3D camera; smooth physics, aim assist, 3 AI strengths |
| **Sliding Puzzle** | 1 player    | 15-puzzle in 3x3/4x4/5x5: slide the numbered tiles into the gap, click or arrow control, score from moves & time |
| **Mastermind**     | 1 player    | Crack the secret colour code (3 modes: 4×6, classic, 5×8), black/white feedback pegs, endless streak high score |
| **Bubble Shooter** | 1 player    | Puzzle Bobble clone: shoot matching colours into groups of three, wall bounces, falling clusters, 3 difficulties |
| **Hangman**        | 1 player    | Guess the word before the gallows is finished; on-screen keyboard, per-language word lists, 3 length modes, endless streak |
| **Block Jump**  | 1 player        | 3D Minecraft-style platformer: textured voxel world, Steve figure, ladders, fences & slime blocks, first/third-person camera, seed-generated parkour levels |
| **Tower Defense** | 1 player      | Fend off endless waves on 4 maps: up to 11 tower types with upgrades, selling & A/B specialisation, bosses, 3 modes, active abilities |
| **Minigolf**    | 1 / 2 players   | 360 holes across 40 courses (18 hand-built, 342 seed-generated): sand, ramps, water, rubber bumpers, windmills & moving blocks; scorecard with par and hole-in-one bonus; **built-in hole editor** with 15 object types, 12 templates and sharing as `.lamapgzmap` |
| **Pinball**     | 1 / 2 players   | Pinball machine with 3 tables: pop bumpers, slingshots, drop targets, L-A-M-A lanes, multiball with jackpot, ball save, nudge & tilt |
| **Bowling**     | 1 / 2 players   | 10 frames with official strike/spare scoring, real pin physics, hook spin and a perspective lane view, 3 difficulties |
| **Crossy Road** | 1 player        | Hop endlessly across grass, roads, rivers and railway tracks in an isometric voxel look: day/night cycle, eagle, 10 unlockable characters, Daily Route |
| **Geometry Dash** | 1 player      | Rhythm platformer with cube, ship, ball, UFO and wave: 16 levels from Easy to Demon with 3 secret coins each, plus a library of **80 more levels** in 10 searchable categories, practice mode, a soundtrack per level; **level editor** with 12 templates and sharing as `.lamapgzlevel` |
| **Battleship**  | 1 / 2 players   | 10x10 naval battle: drag & drop fleet deployment, 3 rule toggles (touching, salvo, fire again), AI with 3 strengths or a local duel with a handover screen |
| **Casino**      | 1 player        | European roulette with all classic bets and the Llama Slot (5 reels, 10 lines, wild, free spins); one llama-chip account shared with Blackjack and Poker |
| **Lama Crossing** | 1 player        | Push-your-luck game with llama chips: lane by lane across a road, rope bridge, river or ice lake, time the gaps in the traffic, cash out after any lane; 4 difficulties up to ×31,784, ~97% return |
| **Lama Mines**  | 1 player        | Push-your-luck game with llama chips: uncover gems, dodge mines - 4x4 to 7x7 fields with 1 to 48 mines, every gem raises the multiplier up to ×100,000, cash out any time; auto mode, SHA-256 fairness check, statistics, ~97% return |
| **Lama Plinko** | 1 player        | Golden balls of yarn bounce down 8 to 16 rows of pegs into multiplier slots: 3 risk levels up to ×1000, up to 50 balls at once, rapid fire and auto mode; fairness check per ball, hit distribution, ~97% return |
| **Lama Crash**  | 1 player        | Crash game with llama chips: the llama rises in a balloon while the multiplier climbs up to ×100,000 - cash out before it pops; two bets with auto target and half cash-out, 6 altitude worlds up to the moon, suspense music, auto mode, fairness check, statistics, ~97% return |

**Multiplayer (2 players local)** is available for **Snake**, **Pong**,
**Air Hockey**, **Tic-Tac-Toe**, **Tetris (Versus)**, **Asteroids (co-op
duel)**, **Memory (duel)**, **Connect Four**, **Tank Duel**, **Reversi**,
**Kniffel**, **Draughts**, **Chess**, **Nine Men's Morris**, **Simon (duel)**,
**Billiards**, **Minigolf**, **Pinball**, **Bowling** and **Battleship** (with a
hidden handover screen) - 20 games in total. The mode is chosen right in the
pre-game screen (*Single-player / Multiplayer*); Tetris additionally offers
**Versus against the AI**. The web version is single-player only.

#### Feature details per game

**Snake**
- **NEW - Replay of the run**: every run is recorded - on game over **P** shows
  the replay (always from above, even if you played in 3D), **S** puts it into
  the archive, **E** shares it as a `.lamapgzreplay` file. Stored are body,
  apples, golden and special apples, score and stamina - in ten-second chapters
  so seeking stays quick.
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
  Everything is stored in `mem.json` (section `ngb`, per profile); all visual personalization runs through the
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
- **NEW - Replay of the game**: every round is recorded - after the results
  screen **P** shows the replay with field, preview, hold, garbage bar and stats
  (both fields in versus), **S** puts it into the archive, **E** shares it as a
  `.lamapgzreplay` file.
- **Modern Guideline rules**: 10x20 field, pieces from a **7-bag**, the **SRS
  rotation system** with real wall kicks (including the I piece), rotation in
  both directions, **hold** (once per piece), a **5-piece preview**, ghost piece
  and **lock delay** (0.5 s, at most 15 resets).
- **Three modes** on the pre-game screen: *Solo*, *Versus AI* and *2 Players*.
  Solo offers **Marathon** (start level 1-15, counts as the high score),
  **Sprint 40 lines** (best time) and **Ultra 2 minutes** (best score) in the
  setup; the bests live in the `tetris` section of `mem.json`.
- **Guideline scoring**: single to Tetris, **T-Spins** (full and mini),
  **Back-to-Back** (x1.5), **combos** and **Perfect Clears** - with callouts,
  line-clear animation, hard-drop trail, particles and a level-up effect.
- **Versus with garbage lines**: cleared lines send garbage to the opponent
  (Tetris = 4, T-Spin Double = 4 …), incoming garbage is announced in a warning
  bar and **cancelled** by your own attacks; both fields get the same piece
  sequence. The **AI** comes in 3 strengths, and the speed rises every 40
  seconds.
- **Controls**: Left/Right with custom **DAS/ARR** (adjustable in the setup),
  Up = rotate right, Down = soft drop, Action = hard drop; **C**/Shift = hold,
  **Z**/**Y** = rotate left, **X** = rotate right. In 2-player mode P1 holds with
  **Q** and rotates left with **E**, P2 with **Right Shift** / **Right Ctrl**.
  After the game: **R** = play again, **S** = setup.

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

**2048**
- **Its own setup screen** with board sizes from **3x3 to 8x8** and three modes:
  *Classic* (goal 2048, then "Keep playing?"), *Time attack* (3 minutes, the
  clock starts with your first move) and *Endless*.
- **Smooth animations**: tiles slide, merge with a pop and grow in; score
  pop-ups, sparks from 128, a shockwave from 2048 and new colours up to 131072.
  Input during an animation is buffered.
- **Undo** (off / 3 per game / unlimited, key **U** or Backspace) - using it
  takes the game out of the high score and the tile achievements.
- **Save & resume**: the running game is saved automatically per size and mode;
  best scores and biggest tile per size/mode are stored in the `g2048` section
  of `mem.json`.
- Controls: arrows/WASD or **swipe** with mouse/touchpad, **R**/**N** = new game,
  **Tab** = setup. The high score only counts in **4x4 Classic** without undo.

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
- **4 variants** with **400 levels** each (4 difficulties x 100): *Classic* (the
  familiar seeded levels - solved levels stay ticked off), *X-Sudoku* (both
  diagonals contain every digit exactly once), *Killer* (dashed cages with a sum;
  400 pre-generated levels) and *Mini 6x6*. Every puzzle has a **unique
  solution** - level 12 of "Hard" is the same puzzle on every PC.
- **Daily Sudoku**: one puzzle per day for everyone, identical on desktop and in
  the browser; the difficulty depends on the weekday (Monday Easy up to Saturday
  Expert), and solving every day builds a streak.
- **Up to 3 stars per level** (solved · no mistakes and no hints · additionally
  under the target time) plus a **best time** in the level select; **puzzles in
  progress** are saved automatically and resumed next time.
- **4 game modes** (chosen before starting) with a score multiplier:
  **Classic** (x2.0 - no assists), **Notes** (x1.5 - + pencil notes and
  automatic candidates), **Comfort** (x1.0 - + wrong digits red, conflicts and
  wrong cage sums marked, correct entries lock in), **Assist** (x0.7 - + hint
  key, max. 3). With the **3-error limit** enabled (setup option) the third
  mistake ends the game.
- Controls: arrows/WASD = cell, **1-9** = digit (numpad too),
  **0/Del/right click** = erase, **U**/**Z** = undo, **Y** = redo, **N** =
  notes, **C** = fill in candidates automatically, **H** = hint, **M** = colour
  marker, **R** = restart level, **Q** = level select. "Digit first" input
  (setup, **I**) and a **remaining-digit counter** under every digit; fully
  playable with the mouse. After the game ends, **A** reveals the full
  **solution**.
- Points = (variant and difficulty base - time - errors - hints) x mode
  multiplier; all variants and the Daily Sudoku count toward the high score.

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
- **Llama chips**: Blackjack plays with the **Llama Bank** account it shares with
  Poker and Casino (start 1000, stored permanently in `mem.json`). The bet is
  taken as soon as the cards are dealt; below 10 chips, Enter takes a **bank
  loan** back up to 1000.
- **High score** = peak of your own **Blackjack balance** (1000 plus everything
  won and lost at Blackjack) - winnings at roulette, the slot or poker don't
  count here, and neither do loans.
- Played via chip buttons and keys (**H**it/**S**tand/**D**ouble/split
  **X**, **1-4** = bet, Backspace = clear bet, Enter = deal) with card
  animations; the dealer's hole card now really flips over when revealed.

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
- Guess the hidden word; colour feedback (green/yellow/grey) with correct
  **duplicate-letter counting** and an on-screen keyboard that colours in (QWERTZ
  for German, Czech, Slovenian and Croatian, AZERTY for French, QWERTY
  otherwise).
- **Four modes**: *Endless* (one word after another with 6 guesses each; every
  solved word scores points, the first unsolved one ends the game), *Daily word*
  (one word a day per language and length - the same on PC and in the browser -
  with countdown and streak; a started daily word is saved), *Dordle* (2 words at
  once in 7 guesses) and *Quordle* (4 words in 9 guesses, the keys show the colours
  of every board).
- **Setup** before every game: **word length 4 to 7**, **hard mode** (revealed
  hints must be reused - green letters stay in place, yellow ones have to appear
  again) and a **colour-blind palette** (orange/blue); your statistics are shown
  next to it.
- **Real word lists in all 14 languages** (folder `woordlistz/`, A-Z only), with
  separate lists per length: for 5 letters alone **33,847 answers** and **213,375
  accepted guesses**, about 134,000 answers across all four lengths. Answers are
  common words without names, English leftovers or offensive words; every guess
  is checked against the list - anything else is rejected and the row shakes
  briefly.
- **Statistics** per language, length and mode: games, win rate, current and best
  streak and the **guess distribution as a bar chart** (`wordle` section of
  `mem.json`). **Share** (**C**) copies an emoji grid to the clipboard without
  giving away the answer.
- The high score only counts *Endless* with 5 letters; the other lengths keep
  their own bests. Achievements **Mind Reader** (2 guesses or fewer), **Word
  Habit** (7 daily words in a row) and **Quad Genius** (solve Quordle).

**Poker**
- **3 variants** on the pre-game screen: **Texas Hold'em** against 1-3 AI
  opponents with dealer button, blinds and four betting rounds, **5 Card Draw**
  (heads-up against the AI, one draw) and **Video Poker** (*Jacks or Better*,
  solo against the paytable).
- Actions via buttons or keys: **F** = fold, **C** = check/call, **R** = raise,
  **A** = all-in; hold/swap cards by clicking or with **1-5**, **Enter** draws or
  deals the next hand.
- **Llama chips** of the shared **Llama Bank**: at the start of a hand your
  account sits on the table as your stack, and whatever goes into the pot is
  taken immediately - leaving the table mid-hand only loses your share of the
  pot. Going broke (below the big blind of 20, below 10 in Video Poker) gives a
  bank loan back up to 1000.
- **High score** = peak of your **Poker balance** (1000 plus all poker wins and
  losses); the **Chip Leader** achievement also counts only your poker balance.

**Chess**
- **Full chess**: every piece move including **castling**, **en passant** and
  **pawn promotion** (choose the piece); **check, checkmate and stalemate** plus
  draws by the **fifty-move rule**, **threefold repetition**, **insufficient
  material** or agreement.
- **Three modes**: a *game* against the AI, *2 players* on one computer (the
  board can flip after every move) and **puzzles**.
- **Stronger, stutter-free AI** in 6 levels from *Beginner* to *Master*:
  iterative deepening, transposition table, quiescence search, an opening book
  and an evaluation with mobility, pawn structure and king safety. The AI thinks
  in small slices per frame - the game never stutters.
- **Setup**: colour choice, **chess clock** (none, 1+0, 3+2, 5+0, 10+5) and
  **Chess960** (all 960 starting positions, the number is shown above the move
  list).
- **Sidebar** with clocks, captured pieces, material balance and a scrollable
  **move list (SAN)**; drag & drop, sliding pieces, coordinates. Keys: **U** =
  undo, **H** = hint arrow, **O** = offer a draw, **X** = resign, **F** = flip
  board, after the game **P** = **PGN export**.
- **Puzzles**: 200 puzzles in 5 stages (mate in 1/2/3, tactics I/II) from the
  free **Lichess puzzle database (CC0)**, verified with the built-in engine; in
  mate puzzles every mating move counts. Progress is stored in the `chess`
  section of `mem.json`.
- Undo and hints mark a game as "assisted": the high score only counts
  unassisted wins against the AI (per session).

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
- **NEW - Replay of the frame**: every shot is recorded as its own sequence -
  with aim, power and the path of all balls. At the end **P** shows the replay
  (in practice mode at any time between two shots), **S** puts it into the
  archive, **E** shares it as a `.lamapgzreplay` file.
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
- **Controls**: the mouse aims, holding the left button loads the power,
  releasing putts, holding the right button locks the power (arrows + space
  work too). **R** cancels a charged shot without putting. **G** toggles the
  aim line, **Z** the auto-aim, **P** the pick-up. **F** resets the current
  hole: strokes back to 0, ball back on the tee - same hole, same course.
- **Power lock (hold the right mouse button)**: freezes the charge bar exactly
  where it stood when you pressed - gold, with the percentage, a padlock and a
  calmly pulsing ring around the ball. That way you can wait for the gap in the
  windmill or for the moving block with the shot fully charged and putt at the
  right moment. Releasing charges on; a locked power even survives the stroke,
  so the next left click putts with exactly that value - a power dialled in
  with up/down arrows can be pinned down the same way.
- **Continue instead of repeat**: at the end of a round the **Next** button
  moves on to the following course (*Classic* → *Pro* → *Tour 1* → *Tour 2* …),
  so the same set of nine holes never comes back; next to it are **Again**
  (same course) and **Setup**. Keys: Enter = continue, R = again, S = setup.
- **Scorecard** on the right with par and strokes per hole; in two-player mode
  each player takes the same hole in turn. Points: 600 per hole, ±300 per stroke
  under/over par, **500 extra for a hole in one**. The lowest stroke count per
  course lives in the `minigolf` section of `mem.json`.
- **Pick-up can be switched off**: by default a hole ends after eight strokes
  and is scored at the minimum. If you would rather keep putting until the ball
  drops, set *Pick up* to OFF in the setup screen (or press **P**).
- **Auto-aim can be switched off**: by default the putter turns towards the cup
  before every stroke. If you would rather line up every hole yourself, set
  *Auto-aim* to OFF in the setup screen (or press **Z**) - the direction you
  last picked then stays put, and on the tee of a new hole the putter points
  neutrally up the course.
- **Replay of the round**: at the end of a round **P** (or the **Replay**
  button) shows the whole round again - shot by shot, with course, HUD and
  scorecard. **S** puts it into the archive (sidebar button **Replays**).
- **Build and share your own holes**: the **MAPS** tab on the setup screen
  leads to your own collection - **New** opens the hole editor. Every hole gets
  a name and an **id** (lower case, no spaces); the id is also the file name
  suggested when you share it. On top of the seven classic obstacles come
  **eight new ones**: pipe (moves the ball to the other end), ice, sticky
  patch, booster, magnet, one-way gate, turntable and jump ramp. The hole size
  is freely adjustable (60x80 up to 160x240), **12 templates** give you a
  starting point, and undo/redo plus **Test** are part of it. **Share** writes
  exactly one hole as a `.lamapgzmap` file - via the save dialog or straight
  into the Downloads folder, with your name as the creator. **Import** reads it
  back in and falls back to `-2` automatically if the id is taken. Hole name,
  id and creator name always run through a **word filter covering all 14
  languages**. A hole is played on its own via **Play**, or the whole
  collection via the fifth course option **Custom**.

**Pinball**
- **NEW - Replay of the game**: one sequence per ball - ball paths (multiball
  included), flipper positions, drop targets, lanes, multiplier and score. At
  the end **P** shows the replay, **S** puts it into the archive, **E** shares it
  as a `.lamapgzreplay` file.
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
- **Replay of the game**: at the end **P** shows every delivery again - ball and
  pins exactly as they were thrown; **S** stores the recording in the archive
  (sidebar button **Replays**).

**Crossy Road**
- **Endless hopping** across grass (trees and rocks block the way), roads with
  cars and trucks, rivers with logs and lily pads, and **railway tracks** where a
  train races in after a warning light and bell - later whole stations with up
  to 5 tracks wait for you. The route is built row by row, always has a walkable
  path, and speed and traffic increase as you go.
- **Isometric voxel look**: characters, vehicles and trees made of shaded blocks
  (pre-rendered per tile size), a smoothly following camera, squash & stretch
  when hopping, water splashes, a flattening animation, feathers and glittering
  coins; from row 50 a **day/night cycle** with headlights.
- **The eagle**: the camera creeps forward - dawdle too long or go back more than
  three rows and the eagle grabs you (a red edge warns you first). Drifting off
  the screen on a log is also game over.
- **Coins & characters**: collected coins (giant coin = 5) are saved and buy new
  characters in the **Characters** tab - frog, pig, penguin, cat, fox, llama,
  robot, ghost and unicorn (25 to 250 coins); the chicken is yours from the start.
- **Modes**: *Endless* (score = furthest row, counts as the high score) and
  *Daily Route* (the same for everyone today, in the browser too, with its own
  daily best). Controls: arrows/WASD, Space/Enter/click = hop forward; in the
  setup **H** = shadows, **N** = day/night. Coins, characters and the daily best
  are stored in the `crossy` section of `mem.json`.

**Geometry Dash**
- **Rhythm platformer**: your icon races to the right on its own - all you decide
  is when to jump or fly. **Five forms** - cube, ship, ball, UFO and wave - plus
  form, gravity and speed portals (0.5x to 3x), yellow/pink/blue **pads and
  orbs**, half blocks, spikes, pits and color triggers.
- **16 built-in levels** from *Easy* to *Demon* ("Lama Inferno", "Lama Eclipse")
  with **3 secret coins** each. Every level is provably beatable: while building
  them a solver beat each one with the real game code - including all coins, and
  even when shifted by 1/240 second.
- **"More" tab: 80 levels in 10 categories** (starter, rhythm, coins, ship, UFO,
  ball, gravity, speed, wave, demon) with a **search field**, a random button and
  its own per-level progress. They are solver-proven as well - goal and all three
  coins. Stars and achievements stay with the campaign.
- **Precise physics**: fixed-point math at a fixed 240 Hz step; every press takes
  effect in exactly the step in which it happened - identical at any frame rate
  and bit-identical in the browser.
- **Practice mode** (**P**) with automatic and manual checkpoints (**Z** places,
  **X** removes), attempt counter, progress bar, explosions and instant restart
  (**R**). Every level has its **own soundtrack** - background, ground and orbs
  pulse to the beat (music can be turned off with **M**).
- **Stars & coins**: beat a level in normal mode to earn its stars, and every
  coin is worth another star; the high score is your **total stars** (135 at
  most). Per-level bests, coins, attempts and jumps are stored in the `geodash`
  section of `mem.json`.
- **Level editor** in the **LEVELS** tab: grid canvas, a palette with 6 groups
  (blocks, hazards, pads & orbs, portals, speed, extras), rotation, undo/redo,
  an overview strip, **test from the start or from here** and level settings
  (start speed, start mode, music style, BPM, colors). The **"verified"** check
  mark only appears once you have beaten your own level. **Share** writes a
  `.lamapgzlevel` file and **Import** reads it back; levels are stored in
  `ugc.json` next to your custom minigolf holes.
- **Templates**: "New" asks for one of **12 templates** first (empty, starter,
  jumps, pads & orbs, ship, wave, ball, UFO, gravity, speed, coins, demon). They
  carry their objects, colors, music and - having been played through by the
  solver - the "verified" check mark. In the **More** tab, **"As template"**
  copies any finished level into your own for rebuilding.

**Battleship**
- **10x10 naval battle** with a carrier (5 cells), battleship (4), cruiser (3),
  submarine (3) and destroyer (2) - whoever sinks the whole enemy fleet first
  wins.
- **Deploy your fleet** by drag & drop from the dock: **R** or right click
  rotates, the preview glows green or red, **X** places everything randomly, **C**
  clears the board; your last layout is suggested again next round.
- **Rules in the setup** (saved): *ships may touch*, *salvo* (as many shots per
  turn as you have ships afloat) and *fire again after a hit*.
- **AI with 3 strengths**: Easy fires at random, Medium hunts down hits
  systematically, Hard computes a **probability map** with checkerboard parity
  (on average about 70 / 60 / 45 shots for a whole fleet). Or **2 players** on
  one computer - a **handover screen** hides both fleets before every turn.
- **Visuals**: radar sweep, animated waves, shells flying in an arc, splashes,
  explosions with smoke and burning cells, a "SUNK!" reveal and an end-of-round
  summary with shots, hits and accuracy. The high score counts your **wins
  against the AI** in one session.

**Casino**
- **Roulette** (European, 37 pockets): every classic bet by clicking a number,
  edge or corner - **straight up** (35:1), split, street, corner, six line,
  column, dozen, red/black, even/odd and low/high. Chip values 1/5/25/100/500,
  right click removes chips; **Spin**, **Rebet** (**R**), **Double** (**D**) and
  **Clear**. The ball spirals into the pocket drawn beforehand, and the last 12
  numbers are shown at the top.
- **Llama Slot**: 5 reels x 3 rows, **10 paylines**, **llama = wild**, **gold
  coins = scatter** with 10 free spins and double wins, line bet 1/2/5/10,
  **auto spin** (10/25), **turbo** and a paytable. The **return to player is
  96.1%** - calculated exactly from the reel strips.
- **Llama Bank**: Casino, Blackjack, Poker, Lama Crossing, Lama Mines, Lama
  Plinko and Lama Crash share one account of **llama chips** (start 1000,
  section `casino` in `mem.json`); old chip counts carry over automatically.
  Bets are taken immediately, every game keeps its own balance for its high
  score, and going broke gives you a **bank loan** back up to 1000.
- Confetti, coin showers, big/mega/jackpot banners and payline animations;
  achievements **Bullseye** (straight-up win at roulette) and **Llama Jackpot**
  (5 llamas on one line).

**Lama Crossing**
- **Push-your-luck game with llama chips**: pick a bet (10 to 10000), press
  **GO** - the llama crosses a road, a canyon, a river or a frozen lake lane by
  lane. Every lane you make raises the **multiplier**, getting caught loses the
  bet. After each lane it's **NEXT** or **CASH OUT**; reaching the other side
  cashes out automatically - with confetti.
- **Obstacles & timing**: on the road cars and trucks drive down every lane, on
  the rope bridge boulders crash through the bridge, on the river driftwood
  floats towards you and on the ice lake ice blocks slide your way. Step out
  while the next lane is occupied and the llama gets run over or hit and lies
  **flat** - so wait for a gap. The lane the llama stands in is protected (on
  the road a barrier drops and the cars stop, elsewhere the obstacles there
  vanish). Higher levels mean denser, faster traffic.
- **4 difficulties**: Easy (22 lanes, risk 1/12, up to ×6.57), Medium (20, 1/6,
  up to ×37.18), Hard (18, 3/10, up to ×595.67) and Extreme (15, 1/2, up to
  ×31,784.96 - every lane doubles). The random risk per lane comes on top of the
  traffic: a sudden **speeder** on the road, a breaking plank, a slipping stone
  or breaking ice. The multipliers are fixed integer tables in
  `lamacrossing_logic.py`, calculated so that **every cash-out strategy returns
  about 97% with clean timing** - no matter when you stop.
- **4 worlds** (buttons top left, key **W**): the **road** seen from above with
  sidewalks and zebra-crossing pieces (default), the **rope bridge** over a
  canyon, **stepping stones in a river** and **ice floes on a frozen lake**. The
  llama wears a scarf in the game's accent colour; the top right shows the
  history of the last 12 rounds.
- **Llama Bank**: the same account as Blackjack, Poker and Casino. The high
  score is the peak of 1000 + Lama Crossing balance, and below 10 chips you get
  a **bank loan** back up to 1000. Leaving mid-crossing costs nothing: bet,
  difficulty and lane are saved, and the next start resumes exactly there.
- Controls: **Space**/**→**/click on the next lane = GO or next lane (once the
  lane is clear), **Enter**/**C** = cash out, **↑**/**↓**/**+**/**-**/mouse
  wheel = bet, **1-4** = difficulty, **W** = world; world, difficulty and last
  bet are stored in the `lamacrossing` section of `settings.json`. Achievements
  **Other Side**, **Nerves of Steel** (cash out at ×10 or more) and **Tightrope
  Walk** (cash out at ×100 or more on Extreme), plus a 3000 milestone and a
  10000 master tier.

**Lama Mines**
- **Push-your-luck game with llama chips**: pick a bet (10 to 10000), press
  **PLAY** - gems and mines are hidden under the covered stone tiles. Every
  **gem** you uncover raises the **multiplier**, a **mine** loses the bet. From
  the first gem on you can **CASH OUT** at any time; find every gem ("ALL
  MINE!") or reach the max win and you are paid out automatically.
- **4 field sizes** (keys **1-4**): 4x4, 5x5 (default), 6x6 and 7x7, with
  **1 to N−1 mines** (**←**/**→** or six quick-pick buttons, default 3) - so up
  to 48 mines and a single gem on 7x7.
- **Multiplier after k gems** (in hundredths, N tiles, M mines):
  `m_k = max(⌊97 · C(N, M) / C(N−k, M)⌋, m_(k−1) + 1)`, capped at ×100,000 -
  integer-only and bit-identical in Python, JavaScript and Rust
  (`lamamines_logic.py`). It never drops below ×1.01 and rises with every gem.
  Since C(N−k, M) / C(N, M) is exactly the chance of finding k gems in a row,
  **every cash-out strategy returns 96.2 to 97%** - no matter when you stop.
  Only three steps with 1 mine on 6x6 and 7x7 are lifted by the floor (at most
  98.9%), and playing all the way to the cap returns less on average. Example,
  5x5 with 3 mines: ×1.10, ×1.25, ×1.44 … up to ×2,231 for all 22 gems. Reach
  ×100,000 and the **max win** is paid out at once.
- **Carry-over**: the hundredths left over when a payout is rounded down are
  added to the next one - small bets lose nothing.
- **Display**: current and next multiplier, the chance that the next tile is
  safe (`(N−M−k) / (N−k)` in %), the win for the next tile and a bar with the
  next 5 steps. Gems come in four colours; the **mascot llama with miner's
  helmet and headlamp** fills its mine cart, cheers at every gem and is covered
  in soot after a mine.
- **Mine hit**: an explosion and camera shake, then the **whole field is
  revealed, dimmed** - so you can see where the other mines were; after cashing
  out as well. A coin shower from ×3, confetti from ×10, on a cleared field and
  on the max win. The top right shows the history of the last 12 rounds.
- **Auto mode** (**A**): mark tiles on the grid in the auto window (click or
  arrows + Space, **Delete**/**Backspace** clears; remembered per field size),
  pick the number of rounds (5/10/25/50/100) and optionally **stop at a profit
  of** or **stop at a loss of** (off/100/500/1000/5000/10000/50000); **Enter**
  starts. Every round uncovers exactly the marked tiles in quick succession and
  cashes out if all of them were safe. Auto runs until the round count is
  reached, a limit kicks in, the chips run out or **STOP**/Esc/Space/Enter or
  **A** is pressed - a round in progress then stays open and can be finished
  by hand. **Turbo** (**T**) cuts all timings to 40%.
- **Fairness check** (**F**): before every round the **hash** is fixed - the
  **SHA-256** of a secret server seed (32 hex characters). The minefield comes
  from the server seed, your own **client seed** (8 characters, re-rolled with
  the **New client seed** button - only outside a round) and the round number:
  randomness via `seedrand` (mulberry32), tiles 0 … N−1 are shuffled and the
  first M are mines. After the round the seed is revealed, and the next round's
  hash is already fixed. The window lists the last 12 rounds (no., hash, server
  and client seed; the mouse wheel scrolls, a click picks a row); **Verify**
  checks the hash and recalculates the minefield - ✓ or ✗ with a mini preview
  of the field.
- **Statistics** (**S**): tiles for rounds, won, lost, wagered, paid out, net
  result, biggest win, best multiplier, **your return**, gems found, mines hit
  and fields cleared, plus a **net-result curve** of the last 100 rounds.
- **Llama Bank**: the same account as Blackjack, Poker, Casino and Lama
  Crossing; PLAY takes the bet and saves the new round in one go
  (`lamabank.bet_and_pay`). The high score is the peak of 1000 + Lama Mines
  balance, and below 10 chips you get a **bank loan** back up to 1000.
  **Resume**: a round in progress is saved with every tile you uncover (seeds,
  round number, field size, mines, bet and the uncovered tiles); on the next
  start the field is recalculated from the seed and the round continues exactly
  there.
- Controls: before a round **Space**/**Enter** = PLAY, **1-4** = field size,
  **←**/**→** = mines, **↑**/**↓**/**+**/**-**/mouse wheel = bet; during a
  round click or **arrow keys** + **Space** = uncover a tile (the first Space
  shows the cursor), **R** = random tile, **Enter**/**C** = cash out; after a
  round Space starts the next one right away. **A** = auto, **S** =
  statistics, **F** = fairness, **T** = turbo, Esc closes an open window or
  stops auto. The keys are fixed and don't depend on the control bindings in
  the options. The round in progress, history, fairness log, statistics,
  carry-over and auto marks are stored per profile in the `casino` section of
  `mem.json`; field size, mines, bet, turbo and the auto settings in the
  `lamamines` section of `settings.json`. Achievements **Treasure Vault** (clear
  a whole field), **Deep Digger** (cash out at ×10 or more) and **Diamond
  Llama** (cash out at ×1000 or more), plus a 3000 milestone and a 10000 master
  tier.

**Lama Plinko**
- **Plinko with llama chips**: pick a bet per ball (1 to 1000), press **DROP** -
  a **golden ball of yarn** bounces down the peg board and lands in a slot with
  a **multiplier**: little in the middle, a lot at the edges. The llama at the
  top of the board nods with every drop.
- **8 to 16 rows** (**←**/**→**, default 12) and **3 risk levels** (**1-3**:
  Low, Medium, High). The outermost slot pays ×16 (Low), ×110 (Medium) or ×1000
  (High) with 16 rows and ×5.3, ×13 or ×32 with 8 rows; the middle pays ×0.7,
  ×0.4 or ×0.2. Rows and risk are locked while balls are still in the air or
  auto is running.
- **27 fixed tables** (9 row counts × 3 risks) in `lamaplinko_logic.py`,
  generated once with `devtools/gen_plinko_tables.py`: symmetric, never falling
  towards the edges, "nice" numbers and each with an **exactly calculated return
  between 96.9 and 97.05%**. A **carry-over** of the hundredths keeps even a bet
  of 1 at about 97%.
- **Result**: n coin flips from the fairness randomness (a random number < 0.5 =
  right), and the slot is the number of right flips - exactly the binomial
  distribution. The animation follows that path peg by peg: arcing bounces, the
  peg that is hit flashes, and a little jitter (cosmetic only) keeps it lively;
  a ball takes 1.62 s (8 rows) to 2.30 s (16 rows). The slot bobs on a hit, a
  popup shows the multiplier and the sound depends on the value; a coin shower
  from ×10, a win banner with confetti from ×100.
- **Lots of balls at once**: at most **50** on the board. **Hold Space** =
  rapid fire at a rate of flight time/30 - that keeps about 30 balls in the
  air; **turbo** (**T**) halves flight time and rate. The bet can be changed
  mid-flight. Hovering over a slot shows its multiplier, probability and the
  win at the current bet; a box next to the board counts the balls, the net
  result and the best hit of the session.
- **Auto mode** (**A**): 10/25/50/100/250/500/1000 balls, optionally with stop
  at a profit or loss of (off/100/500/1000/5000/10000/50000); one ball every
  0.24 s (turbo 0.12 s) with a fixed bet. It runs until the count is reached, a
  limit kicks in, the chips run out or **STOP**/Esc is pressed. In the window:
  **↑**/**↓** = row, **←**/**→** = value, **Enter**/**Space** = START or STOP.
- **Fairness check** (**F**) as in Lama Mines, but **per ball**: hash
  beforehand, seed after the landing, and the window shows the last 12 balls
  (**↑**/**↓**/mouse wheel picks one). **Enter** or **Verify** checks the hash,
  replays the coin flips and shows ✓ or ✗ with a mini preview of the path;
  **C** = new client seed (not while balls are in the air).
- **Statistics** (**S**): the same tiles and net-result curve as in Lama Mines
  (every ball counts as a round), plus the **hit distribution** per row count
  (**←**/**→** switches) as bars against the expected binomial curve.
- **Llama Bank**: the same account as Blackjack, Poker, Casino and Lama
  Crossing. Since the result is already settled when the ball is dropped, it is
  booked right away - bet and win with **one** save (`lamabank.bet_and_pay`),
  together with history, fairness log, statistics and carry-over; chips,
  history and statistics show the win only when the ball lands. **Yarn Storm**
  fires at the drop, **Edge Hit** and **Plinko Legend** on landing - on leaving
  they are caught up for balls still in the air, so nothing gets lost. High
  score = peak of 1000 + Lama Plinko balance; when the account is empty and
  every ball has landed you get a **bank loan** back up to 1000.
- Controls: mouse for everything, **Space**/**Enter**/click on DROP or the
  board = drop (hold = rapid fire), **↑**/**↓**/**+**/**-**/mouse wheel = bet,
  **←**/**→** = rows, **1-3** = risk; **A** = auto, **S** = statistics, **F** =
  fairness, **T** = turbo, Esc closes an open window or stops auto. The keys
  are fixed and don't depend on the control bindings. History, fairness log,
  statistics (with the hit counters) and carry-over are stored per profile in
  the `casino` section of `mem.json`; rows, risk, bet, turbo and the auto
  settings in the `lamaplinko` section of `settings.json`. Achievements **Edge
  Hit** (outermost slot with 12 rows or more), **Yarn Storm** (25 balls on the
  board at once) and **Plinko Legend** (outermost slot with 16 rows on High),
  plus a 3000 milestone and a 10000 master tier.

**Lama Crash**
- **Crash game with llama chips**: pick a bet (10 to 10000), press **PLAY** -
  after a short 3-2-1 the llama rises in a **balloon** and the **multiplier**
  grows with time: ×2 after 11.6 s, ×10 after 38.4 s, ×100 after 76.8 s and
  ×100,000 after 192 s (3.2 minutes). From ×1.01 on you may **CASH OUT** at any
  time - the llama jumps off with a parachute and the win (bet × multiplier)
  is credited at once; if the balloon pops first, the bet is gone. After
  cashing out the empty balloon keeps rising until it pops ("All out – the
  balloon keeps rising"), so you can see how far it would have gone;
  **fast-forward** (Space) runs 16 times as fast.
- **Two bets** in the same round (a second llama in the basket, via the switch
  on card 2 or **Z**), each with its own amount and its own **auto target**
  (×1.01 to ×100,000, **←**/**→**, **X** on/off): when the balloon reaches the
  target, exactly the target is paid - even if it pops in that very moment.
  Each bet can be **halved** once (**H** or HALF for all, **Q**/**W** or the
  button on the card for one): that half floats away as a money sack on a mini
  parachute while the rest keeps climbing.
- **Pop point** from the fairness randomness: a random number r (32 bits), then
  `⌊97 · 2³² / (2³² − r)⌋` in hundredths, at least ×1.00 and at most ×100,000 -
  integer-only and bit-identical in Python, JavaScript and Rust
  (`lamacrash_logic.py`). The chance of reaching ×x is therefore 97% / x
  (rounded down to whole 2⁻³²): **every cash-out target returns 96.997 to
  97.000%**, whether ×1.01 or ×100,000; about 3.96% of rounds pop right at
  ×1.00, before you can cash out. The climb curve is integer-only as well
  (`mu += mu // 1000` per tick of 1/60 s), and the balloon only pops once the
  multiplier **exceeds** the pop point - so cashing out by hand at ×m wins
  under exactly the same condition as an auto target of ×m. Reach ×100,000 and
  you land on the **moon**: the max win for every bet still in the air.
- **Altitude worlds**: llama meadow (pasture, fence, llama barn) → sea of
  clouds (from ×1.5, flocks of birds) → storm front (from ×3, rain, lightning,
  planes) → northern lights (from ×10, dancing auroras) → outer space (from
  ×100, the curve of the Earth, satellites, shooting stars, distant planets -
  and the moon keeps growing) → **moon landing** with a flag, confetti and a
  coin shower. Each new world is announced by name as you reach it, and the
  worlds blend smoothly into one another. The multiplier curve runs on top
  with axes that grow along, and the balloon inflates more and more as it
  climbs.
- **Suspense music** in six stages, one per altitude world
  (`lamacrash_music.py`): from the meadow up to outer space the piece gets a
  whole tone higher and faster with every world (96 to 150 bpm), and the moon
  gets a fanfare; **M** switches the music on or off. From ×2 on the stretched
  balloon squeaks. None of this depends on the pop point - picture and sound
  give nothing away (a twin test in the audit compares two rounds popping at ×5
  and ×50 up to ×4.99, frame by frame and tone by tone).
- **Pop**: a bang, rubber shreds and camera shake; llamas still sitting in the
  basket tumble out of the picture together with it. Cashing out brings a coin
  shower from ×3, confetti from ×10 and a win banner from ×100. The top right
  shows the pop points of the last 12 rounds as coloured pills (red below ×2,
  green from ×2, gold from ×10, purple from ×100).
- **Auto mode** (**A**, between rounds only): 5/10/25/50/100/250 rounds,
  optionally with **stop at a profit of** or **stop at a loss of**
  (off/100/500/1000/5000/10000/50000) and turbo. Every active bet gets an auto
  target, an empty balloon is fast-forwarded automatically, and you can still
  cash out by hand. Auto runs until the round count is reached, a limit kicks
  in, the chips run out or **STOP**/Esc/**A** is pressed - a round in the air
  then flies on normally. In the window: **↑**/**↓** = row, **←**/**→** =
  value, **Enter**/**Space** = START or STOP. **Turbo** (**T**) makes the
  balloon climb three times as fast and shortens the countdown and pauses.
- **Fairness check** (**F**) as in Lama Mines: the **hash** is fixed before
  every round (SHA-256 of the secret server seed), the pop point comes from the
  server seed, your own **client seed** and the round number
  (`fairness.round_rand`, mulberry32), and the seed is revealed after the
  round. The window lists the last 12 rounds (**↑**/**↓** picks one, **C** =
  new client seed while no round is running); **Verify** (**Enter**) checks
  the hash, recalculates the pop point and checks the cash-outs - ✓ or ✗ with a
  mini curve.
- **Statistics** (**S**): tiles for rounds, won, lost, your return, wagered,
  paid out, net result, biggest win, best multiplier, **top pop point**, half
  cash-outs and close calls, plus the **net-result curve** of the last 100
  rounds and the **distribution of pop points** in seven buckets (×1 to
  ×1000+) as bars against the expected distribution (gold dots).
- **Llama Bank**: the same account as Blackjack, Poker, Casino, Lama Crossing,
  Lama Mines and Lama Plinko. PLAY takes the total bet and saves the new round
  in one go (`lamabank.bet_and_pay`), and every cash-out credits the win right
  away; a **carry-over** of the hundredths means even small bets lose nothing.
  **Resume**: leave mid-flight and the next start continues at the same tick
  after a countdown - the pop point is fixed anyway; leave during the empty
  flight and the round is settled right away. If the window loses focus while
  money is in the air, the game **pauses** by itself (new engine hook
  `pause_on_blur` in `game_base.py`, handled by `main.py`). High score = peak
  of 1000 + Lama Crash balance; below 10 chips you get a **bank loan** back up
  to 1000.
- Controls: before a round **Space**/**Enter** = PLAY, **1**/**2**/**Tab** =
  pick a card, **↑**/**↓**/**+**/**-**/mouse wheel = bet, **←**/**→** or the
  mouse wheel over the target = auto target, **X** = target on/off, **Z** =
  second bet; in flight **Space**/**Enter**/**C** = everyone out, **1**/**2** =
  one bet, **H** = half, **Q**/**W** = half of one bet; afterwards **Space** =
  fast-forward. **A** = auto, **S** = statistics, **F** = fairness, **T** =
  turbo, **M** = music, Esc closes a window or stops auto (otherwise pause).
  The windows only open while no bet is in the air; the keys are fixed and
  don't depend on the control bindings. The round, history, fairness log,
  statistics and carry-over are stored per profile in the `casino` section of
  `mem.json`; bets, targets, the second bet, turbo, music and the auto
  settings in the `lamacrash` section of `settings.json`. Achievements **Close
  Call** (cash out by hand at ×2 or more, at most 5% before the pop), **Aurora
  Pilot** (cash out at ×10 or more) and **Space Llama** (cash out at ×1000 or
  more), plus a 3000 milestone and a 10000 master tier.

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
  the previous record as a chip. With many modes at a small resolution it
  becomes **compact**: Options, Matches, Wiki and Back move into one row and the font
  adapts - nothing runs off-screen any more.
- **Unified in-game look**: all 50 games share the menu's theme palette and
  font - HUDs, setup screens and overlays follow the design chosen in the
  options (v4.1 / v4 / Classic), while each playfield keeps its identity
  colours. Every game now handles mid-game resolution changes cleanly, and
  menu names are language-aware (e.g. "Schach" → "Chess" / « Échecs »).
- **In-game wiki** ("LamaWiki"): detailed help for every game (controls, modes,
  scoring, tips) plus general pages - with a **search box**, categories,
  scrollable articles and keycap chips, in all 14 languages. Reachable via
  the **"Wiki / Help"** sidebar button and from every game's pre-game screen
  (opens that game's page directly).
- **Achievements, rarity & level**: **274 achievements** in five categories -
  60 collection-wide goals (rounds, playtime, records, wins, days played, daily
  streaks, duels …), 41 **score milestones**, 34 **master tiers** (the second,
  much higher mark), a **regular** achievement for each of the 50 games (25
  rounds) and 89 **special moments** (checkmate, a Yahtzee, the 4096 tile, a
  hole-in-one, an air hockey hat-trick, four ghosts on one power pill …). Every
  achievement has a **rarity** (common 10 up to mythic 300 points); the points
  make your **level** with 14 **ranks** from Llama Chick to Star Llama.
  Unlocking shows a pop-up in the rarity's colour, a level-up its own one with
  a fanfare.
- **Statistics with charts**: five tabs - Achievements (with filter, points and
  XP bar), Statistics (cards, top-10 bars, win/loss ring, table), **Calendar**
  (days-played map, streaks, weekdays, time of day), **History** (everything
  with a timestamp, plus each game's score curve) and **Profiles** (device
  leaderboard, head-to-head record). On top of that the **PyGameZ recap** with
  your best day, favourite time and longest session. It all lives in the file
  **`stats.pgzlamastats`** and can be exported, shared and read back in - also
  between the desktop and the web version.
- **Player profiles**: several players on one device, each with their own high
  scores, achievements, level, statistics, chips and saved games (up to eight).
  The **"Profiles"** button creates, renames and switches; in multiplayer you
  pick player 2's profile before the match (or play as a guest), the names then
  replace "Player 1/2" in game, and every match goes into the
  **head-to-head record**.
- **Replays**: **six games** record their rounds - **Minigolf**, **Bowling**,
  **Billiards**, **Pinball**, **Snake** and **Tetris**. What is stored is the
  actual motion, frame by frame (30 samples per second), not the input; the
  replay therefore looks exactly like the round you played, HUD and scorecard
  included. In Minigolf, Bowling, Billiards and Pinball a sequence is a stroke,
  roll, shot or ball; Snake and Tetris run on and are cut into ten-second
  **chapters** (key frame + changes, so seeking stays quick). At the end of a
  round **P** shows the replay and **S** puts it into the archive; the sidebar
  button **"Replays"** opens it again at any time (one tab per game, pause, jump
  to the next sequence, speed 0.5x to 4x, clickable progress bar, delete with
  Del). The archive holds 20 recordings per game; it can be switched off on the
  first start and in the options.
- **Sharing replays**: **E** writes a recording as a **`.lamapgzreplay`** file
  (from the archive as well as right after the round), **I** reads such a file
  back in - it lands in the tab of the matching game and carries a small arrow
  there. One file holds exactly one replay and travels from machine to machine
  like a minigolf course or a Geometry Dash level; recordings that are already
  in the archive are not imported twice. The **browser version** (`web/`) does
  all of this as well - and the files are **interchangeable**: a round recorded
  on the PC plays in the browser and vice versa. There the archive lives in
  browser storage, so it holds 5 recordings per game.
- **Match history**: every one of the 50 games remembers its matches. The **"Matches"** button in
  the pre-game screen opens **Last matches** (a curve of the last 30 results, records in gold, plus the
  last 10 matches) and **All matches** (mode filter **M**, newest/best sorting **O**). Saved are date,
  mode, points or chip profit, win/loss/draw, duration and details such as bet, multiplier, coins or
  level - the last **250** matches per game (the totals keep counting) in
  **`matches.pgzlamamatches`**, separately for each profile. The same in the web version, the Rust
  build and on pygamez.com.
- **Daily archive**: the daily track (Crossy Road), daily puzzle (Sudoku) and daily word (Wordle) can
  be replayed for the **last 30 days** and every day you have already played - with the same seed as
  back then and a separate best score or best time per day. In the setup, the arrows next to the date
  or **Page up/down** change the day, **K** opens the calendar. Archive days don't change streaks or
  daily statistics; storage is compact (one day number instead of a date - a year of Crossy archive is
  about 4 KB).

### Controls

- Pick a game via the button in the menu on the left. A **pre-game screen** then
  appears: choose **Single-player** or **Multiplayer**, go to the **options** or
  back. Arrows/mouse to select, Enter to start.
- **ESC** = pause / resume (in menus: back). Games that would otherwise carry
  on without you pause by themselves when the window loses focus (engine hook
  `pause_on_blur` - so far Lama Crash, while a bet is in the air).
- **F11** (or the "Fullscreen on/off" button) = toggle fullscreen. The Pygame
  display stays embedded and is scaled up keeping its aspect ratio (black bars
  when the aspect ratio differs). The window can be resized freely.
- **"Back to menu"** ends the game and saves the high score - and so does
  switching to another game from the sidebar.
- **Fixed extra keys**: besides the five bindable actions, some games have keys
  of their own (e.g. hold **C** and rotate left **Z** in Tetris, undo **U** in
  2048, Chess and Sudoku). They only work if that key isn't bound to an action in
  the options, and they are listed in the setup hint and the wiki. Held keys are
  detected cleanly and released on pause or Alt-Tab - nothing gets "stuck".
- **"Quit"** closes Pygame and Tkinter cleanly.

### Options, controls & sound

The options screen opens via the **"Options / Controls"** button (on the left) or
from the pre-game screen. It is organized into **three tabs**
(**General / Controls / Appearance**; switch by clicking or with the Tab key):

- **General**: **sound** on/off, **volume** and **haptics** (gamepad vibration,
  only effective with a connected controller) plus **auto resolution**,
  **resolution**, **FPS** and **language** – each toggled with Left/Right.
  Under **Recording** sits **Record replays**: off means that Minigolf and
  Bowling capture nothing.
- **Controls**: **presets** (*WASD + Arrows*, *WASD + IJKL*, *Arrows + WASD*)
  and **rebind every single key** for player 1 and player 2: select a row,
  press Enter, press the desired key (Esc cancels).
- **Appearance**: pick the **UI design** – **UI v4.2** (default: Midnight
  Glass – a deep midnight gradient with slowly drifting glows in indigo,
  turquoise and magenta, fine film grain, sparse stars and panels like
  frosted glass with a bright top edge), **UI v4.1** (like UI v4 but livelier
  – subtle stars plus Saturn and a black hole in the start screen's
  background), **UI v4.1.1** (like v4.1, but a tiled **zigzag pattern** in
  black and charcoal instead of the starfield), **UI v4.1.2** (the same
  pattern in the palette's blues – accent blue as the dominant colour, a
  darker blue as the base), **UI v4.1.3** (the same pattern in UI v4's indigo
  accent on black), **UI v4.1.4** (in UI v4's graphite tone on black),
  **UI v4** (a completely calm, flat graphite look with a single indigo
  accent), **UI v3** (the previous classic UI with starfield, aurora lights
  and glow effects), **UI v2** (the very first UI rework: navy gradient with
  starfield, buttons with a static glow and arrow, titles with a drop shadow
  – no animations at all) or **UI v1** (the look before the UI rework: plain
  dark background, flat buttons, no effects). All cards show a small preview;
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
start.command        Launch script for macOS (double-click in Finder, calls start.sh)
pyinstall-dev/
  pyinstall.bat      EXE build (Windows): bundles everything into builds\PyGameZ.exe
main.py              Tkinter UI, Pygame embedding, central game loop
game_base.py         Game base class (update/draw/handle_event) + InputEvent + helpers
settings.py          Load/save settings (sound/haptics/key bindings/game options with validation rules) (JSON)
audio.py             Procedural sound effects, music loops + gamepad rumble
menu.py              Language, pre-game (mode) and options screen (sound/controls)
ui.py                Shared UI toolkit for all Pygame screens: ten themes, background, titles, buttons, panels, effects
logo.py              Central logo handling: which logo from logo/ is loaded (window icon, menu)
highscore.py         Load/save high scores (section in mem.json)
store.py             Central save file mem.json (device-wide: mem; per profile: highscores, achievements + game progress), atomic with .bak backup
profiles.py          Player profiles: create, switch, rename, name check, colour/icon
stats.py             Statistics in stats.pgzlamastats: totals, days played, time of day, score curves, history, duels, export/import
achievements.py      Achievements: definitions, rarity tiers, unlock logic, toast overlay
levels.py            Level system: achievement points -> level and 14 ranks
charts.py            Charts for the statistics screens (bars, lines, calendar heatmap, rings, tooltip)
progress.py          Achievements & statistics screen (five tabs, scrollable)
profileview.py       Profile screen, player 2 picker and the starting-level card
replay.py            Recording, archive & sharing of replays (replay.json, .lamapgzreplay)
replayview.py        Replay screen: archive list, playback, share/import
matches.py           Match history per game and profile (matches.pgzlamamatches): entries, totals, cap 250
matchview.py         Matches screen: last matches (curve + last 10) and all matches (filter, sorting)
dailyarchive.py      Daily archive: last 30 days + played days, compact storage, calendar window
ugc.py               Custom content (minigolf holes, Geometry Dash levels): storage, checks, export/import (ugc.json)
swear.py             Word filter for names and ids (lang/swear/*.yml, all 14 languages)
filepick.py          File dialogs ("Export as ...", "Import") and the Downloads folder
prestige.py          Prestige system for Snake
competitive.py       Tuning for Snake's Competitive mode (levels, slot machine, gamble apples)
ngb.py               Visual personalization ("mods"): head color + coordinate grid + menu (section ngb per profile)
lamabank.py          Llama Bank: shared chip account of Blackjack, Poker, Casino, Lama Crossing, Lama Mines, Lama Plinko and Lama Crash (section casino in mem.json)
seedrand.py          Random generator with bit-identical numbers in Python and the browser (daily modes, new puzzles)
fairness.py          Fairness check of Lama Mines, Lama Plinko and Lama Crash: server/client seed, nonce, SHA-256 hash, verify (counterparts: web/js/games/fairness.js, rust/crates/pgz-core/src/fairness.rs)
chipstats.py         Statistics of Lama Mines, Lama Plinko and Lama Crash (rounds, wagered, paid out, return, net-result curve) for the S window (plus chipstats.js, chipstats.rs)
i18n.py              Translation engine (loads lang/*.json, t("key"))
lang/
  de.json  en.json  fr.json  es.json  pt.json   Language strings (one placeholder key per text)
  lang.expansion/                           pl, tr, da, no, sv, fi, cs, sl, hr
  swear/                                    Word-filter lists per language (regex, .yml)
lamawiki/
  lamawiki.py          In-game wiki (search, categories, article renderer)
  de.json  en.json  fr.json  es.json  pt.json   Wiki content (one page per game + general pages)
  lang.expansion/                           pl, tr, da, no, sv, fi, cs, sl, hr
woordlistz/
  build_wordlists.py   Rebuilds the Wordle word lists from dictionaries + frequency lists
  de/ en/ fr/ ... hr/  answers.txt + allowed.txt (5 letters), answers4/6/7.txt + allowed4/6/7.txt (4, 6, 7 letters), 14 languages
devtools/            Developer tools (not bundled into the .exe)
  merge_staging.py           Merges translations and wiki pages from devtools/staging/ into all 14 language files
  build_chess_puzzles.py     Builds the 200 chess puzzles from the Lichess puzzle database (CC0)
  build_sudoku_killer.py     Generates the 400 uniquely solvable Killer Sudokus
  build_crossyroad_models.py Writes the Crossy Road voxel models for the web version
  build_geodash_levels.py    Builds the 16 Geometry Dash campaign levels and proves with the solver that each is beatable, coins included
  build_geodash_extra.py     Builds the library (80 levels in 10 categories) and the 12 templates, each proven by the solver
  build_geodash_solver.py    Solver using the real step code (solutions in geodash_proofs.json)
  geodash_sections.py        Section kit for build_geodash_extra.py: short level pieces that can be strung together
  gen_plinko_tables.py       Generates the 27 Plinko tables (8-16 rows × 3 risks) with an exactly calculated return for lamaplinko_logic.py
  export_chiplogic_ref.py    Writes the reference values of the chip rules (fairness, statistics, Mines, Plinko, Crash) for the Rust unit tests
  export_achievements.py     Writes the achievement and level definitions for the web version (web/js/core/achdefs.js)
games/
  snake.py  pong.py  airhockey.py  tictactoe.py  breakout.py
  tetris.py  tetris_core.py  tetris_ai.py
  invaders.py  asteroids.py  pacman.py  flappy.py  doodle.py
  game2048.py  minesweeper.py  sudoku.py  sudoku_gen.py  sudoku_draw.py
  frogger.py  memory.py  solitaire.py  cards.py  aimtrainer.py
  connect4.py  tanks.py  blackjack.py  tunnelracer.py
  labyrinth.py  maze_gen.py  reversi.py  kniffel.py  wordle.py  wordle_words.py
  trexrunner.py  dame.py  poker.py  muehle.py
  chess.py  chess_engine.py  chess_draw.py
  simon.py  billiard.py  slidepuzzle.py  mastermind.py
  bubbleshooter.py  hangman.py  hangman_words.py  blockjump.py
  lamatowerdefense.py  minigolf.py  minigolf_gen.py
  minigolf_draw.py  minigolf_edit.py  pinball.py  bowling.py
  crossyroad.py  crossyroad_world.py  crossyroad_draw.py
  geodash.py  geodash_core.py  geodash_draw.py  geodash_edit.py  geodash_extra.py  geodash_music.py
  battleship.py  battleship_core.py
  casino.py  casino_logic.py  casino_roulette.py  casino_slots.py  casino_draw.py
  lamacrossing.py  lamacrossing_logic.py  lamacrossing_draw.py
  lamamines.py  lamamines_logic.py  lamamines_draw.py
  lamaplinko.py  lamaplinko_logic.py  lamaplinko_draw.py
  lamacrash.py  lamacrash_logic.py  lamacrash_draw.py  lamacrash_world.py  lamacrash_music.py
  levels/              Level data: snake-comp.json, chess-puzzles.json (+ source README), sudoku-killer.json, geodash.json, geodash_extra.json, geodash_templates.json
tests/
  arcade_casino_audit.py     Overall audit (input, seedrand Python = JS, saving, language files, pre-game screens) + all audit_*.py
  audit_tetris.py  audit_2048.py  audit_chess.py  audit_sudoku.py  audit_wordle.py
  audit_crossyroad.py  audit_geodash.py  audit_battleship.py  audit_casino.py  audit_lamacrossing.py   Headless audits per game
  audit_lamamines.py  audit_lamaplinko.py  audit_lamacrash.py  audit_chiplogic.py   Lama Mines, Lama Plinko, Lama Crash and the shared chip logic (fairness, statistics, Python = JS = Rust)
  newgames_audit.py  blockjump_audit.py  progress_audit.py  replay_audit.py
rust/                The Rust version (standalone .exe, see rust/README.md)
  ENGINE.md            Porting contract: how a game moves from Python to Rust
  crates/pgz-core/     Data layer without graphics (store, settings, i18n, profiles,
                       highscore, levels, stats, achievements, seedrand, swear)
  crates/pgz-engine/   Window, drawing, input, sound, Game trait (ui.py + audio.py + game_base.py)
  crates/pgz-games/    The ported games
  crates/pygamez/      The program: menu, sidebar, screens
  tools/export_defs.py Generates the master data (50 games, 274 achievements, 10 themes) from the Python sources
  tools/audit.py       Test bench: translations, game list, conventions
rust.output/         Build output of the Rust version (rust.output/release/pygamez.exe)
```

The chosen language is stored in `mem.json` (in the `mem` section, next to the
`highscores` section in the same file) and loaded automatically on the next launch.

**Sources & licences:** the 200 chess puzzles come from the
[Lichess puzzle database](https://database.lichess.org/#puzzles) (licence
**CC0 1.0**, public domain - thank you, lichess.org!); details are in
`games/levels/chess-puzzles.README.md`. The sources of the Wordle word lists are
listed in `woordlistz/README.md`.

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

#### macOS with start.command

1. **Install Python**, ideally **Python 3.13 from
   <https://www.python.org/downloads/macos/>** (the installer includes Tkinter).
   Homebrew works too: `brew install python python-tk` (without `python-tk`,
   Tkinter is missing). Apple's preinstalled `/usr/bin/python3` is not enough (old Tk 8.5).
2. Get the project folder (`git clone` or ZIP) and **double-click `start.command`**.
   On first launch the script creates `.venv` and installs pygame-ce (about a
   minute), then the collection starts.
3. If macOS blocks it ("unidentified developer"):
   **System Settings → Privacy & Security → "Open Anyway"**.

In the terminal, `./start.sh` does the same. If it says "Permission denied" (can
happen after unpacking a ZIP), use `bash start.sh` or run
`chmod +x start.sh start.command` once.

#### Linux with start.sh

```bash
sudo apt install python3 python3-venv python3-tk   # once (Debian/Ubuntu)
./start.sh                                         # creates .venv, installs pygame, starts
```

Fedora: `sudo dnf install python3 python3-tkinter`, Arch: `sudo pacman -S python tk`.
If `start.sh` can't find a suitable Python, it prints the right command.

#### Using a different Python version

`install-python.bat` sets up Python 3.13 by default. If you prefer 3.12 (or another
version), change the line `set "PYVER=3.13"` in the file to the desired version
and the winget ID accordingly (`Python.Python.3.12`).

#### Building a standalone EXE (Windows)

```bat
pyinstall-dev\pyinstall.bat   :: builds builds\PyGameZ.exe (everything in one file)
```

`pyinstall.bat` uses the `.venv` (creating it if needed), automatically
installs **PyInstaller** and bundles the complete game - Python, pygame, all
games, languages, wiki and logos - into **a single `PyGameZ.exe`** in the
**`builds\`** folder. The file runs on any Windows PC without Python
installed and can be copied freely. Settings and high scores
(`settings.json`, `mem.json`) are created next to the .exe
while playing.

#### Troubleshooting

- **`pygame` not found** → is the venv activated? Repeat step 3
  (`pip install -r requirements.txt`).
- **`python` not recognized (Windows)** → Python was installed without "Add to
  PATH"; reinstall and tick the box, or use `py` instead of `python`.
- **No sound** → check "Sound" in the options; haptics only work with a controller.
- **Window/embedding on Linux** → see *Platform notes* (Wayland/XWayland).

<div align="right"><b><a href="#pygamez">↑ back to top / nach oben</a></b></div>
