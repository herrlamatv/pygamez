# PyGameZ

**🌐 Sprache / Language:** **🇩🇪 [Deutsch](#-deutsch)** · **🇬🇧 [English](#-english)** <br> Game Available in : German, English, French

---

<a name="-deutsch"></a>

## 🇩🇪 Deutsch

Eine Desktop-Spielesammlung in Python: **Tkinter** bildet Fenster und Menü,
**Pygame** wird als Spiel-Display in das Tkinter-Fenster eingebettet. Dreizehn
Spiele mit gemeinsamen Optionen, frei belegbarer Steuerung, Highscores,
prozeduralen Soundeffekten und teilweise Mehrspieler-Modus. Die Oberfläche ist
**mehrsprachig** (Deutsch / English); die Sprache wird beim ersten Start gewählt
und lässt sich jederzeit in den Optionen umstellen.

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

**Mehrspieler (2 Spieler lokal)** gibt es für **Snake**, **Pong**, **Air Hockey**,
**Tic-Tac-Toe**, **Tetris (Versus)** und **Asteroids (Koop-Duell)**. Der Modus wird
direkt im Vorspiel-Screen (*Einzelspieler / Mehrspieler*) gewählt.

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
  **Lila Äpfel** setzen **50 % der Länge** aufs Spiel und multiplizieren diesen Teil
  zufällig mit **x0.5 .. x2.0** - man schrumpft so auf bis zu 75 % oder wächst auf
  bis zu 150 %. Es gibt **15 Level** (Multiplikator bis x16, bis zu 16 Äpfel
  gleichzeitig); die Stufen stehen in `games/levels/snake-comp.json` und lassen sich
  dort ohne Code-Änderung erweitern, die übrige Feineinstellung in `competitive.py`.
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

Highscores werden im Abschnitt `highscores` von `mem.json` (neben dem Code)
gespeichert – gemeinsam mit der Sprache (Abschnitt `mem`).

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
öffnet sich der Options-Bildschirm:

- **Sound** an/aus, **Lautstärke** und **Haptik** (Gamepad-Vibration, nur mit
  angeschlossenem Controller wirksam) – jeweils per Links/Rechts umschalten.
- **Vorlagen** für die Steuerung: *WASD + Pfeile*, *WASD + IJKL*, *Pfeile + WASD*.
- **Jede einzelne Taste** für Spieler 1 und Spieler 2 frei belegen: Zeile wählen,
  Enter drücken, gewünschte Taste drücken (Esc bricht ab).

Einstellungen werden dauerhaft in `settings.json` gespeichert. Im **Einzelspieler**
steuern beide Belegungen dieselbe Figur (Standard: WASD *und* Pfeile), im
**Mehrspieler** je eine. Alle Spiele haben **Soundeffekte** (prozedural erzeugt,
keine Extra-Dateien nötig), die sich global stummschalten lassen.

### Projektstruktur

```
install-python.bat  Windows-Einrichtung: Python 3.13 + .venv + pygame
start.bat            Startskript (Windows)
start.sh             Startskript (Linux / macOS / Git Bash)
main.py              Tkinter-Oberfläche, Pygame-Einbettung, zentrale Game-Loop
game_base.py         Game-Basisklasse (update/draw/handle_event) + InputEvent + Helfer
settings.py          Einstellungen (Sound/Haptik/Tastenbelegung) laden/speichern (JSON)
audio.py             Prozedurale Soundeffekte + Gamepad-Rumble
menu.py              Sprach-, Vorspiel- (Modus) und Options-Screen (Sound/Steuerung)
highscore.py         Laden/Speichern der Highscores (Abschnitt in mem.json)
store.py             Zentrale Speicherdatei mem.json (Abschnitte: mem, highscores)
prestige.py          Prestige-System für Snake
competitive.py       Kennzahlen für den Competitive-Modus von Snake (Level, Slot, Wett-Äpfel)
ngb.py               Visuelle Personalisierung ("Mods"): Kopffarbe + Koordinaten-Raster + Menü (mem-ngb.json)
i18n.py              Übersetzungs-Engine (lädt lang/*.json, t("schlüssel"))
lang/
  de.json  en.json   Sprach-Strings (ein Platzhalter-Schlüssel je Text)
games/
  snake.py  pong.py  airhockey.py  tictactoe.py  breakout.py  tetris.py
  invaders.py  asteroids.py  pacman.py  flappy.py  doodle.py
  game2048.py  minesweeper.py
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
#   oder:  pip install "pygame>=2.6"

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
**Pygame** is embedded as the game display inside the Tkinter window. Thirteen
games with shared options, freely rebindable controls, high scores, procedural
sound effects and, for some titles, a multiplayer mode. The interface is
**multilingual** (German / English); the language is chosen on first launch and
can be changed at any time in the options.

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

**Multiplayer (2 players local)** is available for **Snake**, **Pong**,
**Air Hockey**, **Tic-Tac-Toe**, **Tetris (Versus)** and **Asteroids (co-op
duel)**. The mode is chosen right in the pre-game screen
(*Single-player / Multiplayer*).

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
  matching symbols). **Purple apples** put **50 % of your length** on the line and
  multiply that part by a random **x0.5 .. x2.0** - so you shrink to as little as 75 %
  or grow up to 150 %. There are **15 levels** (multiplier up to x16, up to 16 apples
  at once); the levels live in `games/levels/snake-comp.json` and can be extended there
  without touching code, the rest of the tuning lives in `competitive.py`.
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

High scores are stored in the `highscores` section of `mem.json` (next to the
code) – together with the language (section `mem`).

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
from the pre-game screen:

- **Sound** on/off, **volume** and **haptics** (gamepad vibration, only effective
  with a connected controller) – each toggled with Left/Right.
- **Presets** for the controls: *WASD + Arrows*, *WASD + IJKL*, *Arrows + WASD*.
- **Rebind every single key** for player 1 and player 2: select a row, press
  Enter, press the desired key (Esc cancels).

Settings are stored permanently in `settings.json`. In **single-player** both
bindings control the same character (default: WASD *and* arrows), in
**multiplayer** one each. All games have **sound effects** (procedurally
generated, no extra files needed) that can be muted globally.

### Project structure

```
install-python.bat  Windows setup: Python 3.13 + .venv + pygame
start.bat            Launch script (Windows)
start.sh             Launch script (Linux / macOS / Git Bash)
main.py              Tkinter UI, Pygame embedding, central game loop
game_base.py         Game base class (update/draw/handle_event) + InputEvent + helpers
settings.py          Load/save settings (sound/haptics/key bindings) (JSON)
audio.py             Procedural sound effects + gamepad rumble
menu.py              Language, pre-game (mode) and options screen (sound/controls)
highscore.py         Load/save high scores (section in mem.json)
store.py             Central save file mem.json (sections: mem, highscores)
prestige.py          Prestige system for Snake
competitive.py       Tuning for Snake's Competitive mode (levels, slot machine, gamble apples)
ngb.py               Visual personalization ("mods"): head color + coordinate grid + menu (mem-ngb.json)
i18n.py              Translation engine (loads lang/*.json, t("key"))
lang/
  de.json  en.json   Language strings (one placeholder key per text)
games/
  snake.py  pong.py  airhockey.py  tictactoe.py  breakout.py  tetris.py
  invaders.py  asteroids.py  pacman.py  flappy.py  doodle.py
  game2048.py  minesweeper.py
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
#   or:  pip install "pygame>=2.6"

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

#### Troubleshooting

- **`pygame` not found** → is the venv activated? Repeat step 3
  (`pip install -r requirements.txt`).
- **`python` not recognized (Windows)** → Python was installed without "Add to
  PATH"; reinstall and tick the box, or use `py` instead of `python`.
- **No sound** → check "Sound" in the options; haptics only work with a controller.
- **Window/embedding on Linux** → see *Platform notes* (Wayland/XWayland).

<div align="right"><b><a href="#pygamez">↑ back to top / nach oben</a></b></div>