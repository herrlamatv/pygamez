# PyGameZ

Eine Desktop-Spielesammlung in Python: **Tkinter** bildet Fenster und Menü,
**Pygame** wird als Spiel-Display in das Tkinter-Fenster eingebettet. Sieben
Spiele mit gemeinsamen Optionen, frei belegbarer Steuerung, Highscores,
prozeduralen Soundeffekten und teilweise Mehrspieler-Modus. Die Oberfläche ist
**mehrsprachig** (Deutsch / English); die Sprache wird beim ersten Start gewählt
und lässt sich jederzeit in den Optionen umstellen.

## Schnellstart

### Windows

```bat
install-python.bat    :: einmalig: Python 3.13 + .venv + pygame einrichten
start.bat             :: Spielesammlung starten
```

### Linux / macOS / Git Bash

```bash
./start.sh            # startet mit .venv, sonst System-python3
```

`start.bat` / `start.sh` verwenden automatisch die virtuelle Umgebung `.venv`,
falls vorhanden, sonst das System-Python. Eine ausführliche Schritt-für-Schritt-
Anleitung steht ganz unten unter **[Installations-Guide](#installations-guide)**.

## Die Spiele

| Spiel        | Modi            | Kurzbeschreibung |
|--------------|-----------------|------------------|
| **Snake**    | 1 / 2 Spieler   | Deluxe-Snake mit Boost, 5 Spielmodi, Goldäpfeln und Prestige |
| **Pong**     | 1 / 2 Spieler   | Klassiker gegen KI oder Spieler 2, umschaltbarer Bewegungsmodus |
| **Tic-Tac-Toe** | 1 / 2 Spieler | m,n,k-Spiel auf 3x3 bis 9x9, drei KI-Stärken **oder** lokal X gegen O |
| **Breakout** | 1 Spieler       | Brick-Breaker mit Steinsorten, Power-ups, Combos und vielen Levels |
| **Tetris**   | 1 / 2 Spieler   | Klassik oder Versus (zwei Felder nebeneinander) |
| **Invaders** | 1 Spieler       | Space Invaders: Wellen leeren, Leben schützen |
| **2048**     | 1 Spieler       | Zahlen-Schiebespiel, Ziel: die 2048er-Kachel |

**Mehrspieler (2 Spieler lokal)** gibt es für **Snake**, **Pong**,
**Tic-Tac-Toe** und **Tetris (Versus)**. Der Modus wird direkt im Vorspiel-Screen
(*Einzelspieler / Mehrspieler*) gewählt.

### Feature-Details je Spiel

**Snake**
- **Boost**: Boost-Taste **gedrückt halten** = Turbo (doppeltes Tempo), verbraucht
  Ausdauer (Balken); ist sie leer, schaltet der Boost ab und lädt sich wieder auf.
  Standard P1 = Leertaste/Shift-links, P2 = Enter/Shift-rechts.
- **5 Spielmodi** (im Setup wählbar): *Klassisch*, *Speed-Rush* (wird mit jedem
  Apfel schneller), *Hindernisse* (tödliche Blöcke), *Portale* (Teleporter-Paare),
  *Zeitangriff* (60 Sekunden, so viele Äpfel wie möglich).
- **Goldäpfel** (zeitweise) geben viele Punkte und füllen den Boost sofort auf.
- Optionale **Wände-durchgehen**, Bonus-Äpfel, **Prestige** (Einzelspieler, Taste **P**).
- Optik: abgerundete Schlange mit Augen, Boost-Glow, Partikel.

**Pong**
- Einzelspieler gegen KI, Mehrspieler = Spieler 2 rechts. Bis 5 Punkte.
- **Bewegungsmodus je Steuerung umschaltbar**: *Dauer* (einmal drücken -> fährt
  weiter, Standard) oder *Halten* (bewegt nur solange gedrückt).
  Umschalten: **X** = Steuerung 1, **N** = Steuerung 2 (wird in `settings.json` gemerkt).
- Ball-Physik mit Beschleunigung und Winkel je nach Treffpunkt.

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

**2048** – Pfeile/WASD schieben alle Kacheln; gleiche Zahlen verschmelzen.

Highscores werden im Abschnitt `highscores` von `mem.json` (neben dem Code)
gespeichert – gemeinsam mit der Sprache (Abschnitt `mem`).

## Bedienung

- Spiel links im Menü per Button wählen. Danach erscheint ein **Vorspiel-Screen**:
  **Einzelspieler** oder **Mehrspieler** wählen, zu den **Optionen** gehen oder
  zurück. Pfeile/Maus zum Wählen, Enter startet.
- **ESC** = Pause / weiter (in Menüs: zurück).
- **F11** (oder Button „Vollbild an/aus") = Vollbild ein/aus. Das Pygame-Display
  bleibt eingebettet und wird seitenverhältnistreu hochskaliert (schwarze Ränder
  bei abweichendem Seitenverhältnis). Das Fenster lässt sich frei skalieren.
- **„Zurück zum Menü"** beendet das Spiel und speichert den Highscore.
- **„Beenden"** schließt Pygame und Tkinter sauber.

## Optionen, Steuerung & Sound

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

## Projektstruktur

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
i18n.py              Übersetzungs-Engine (lädt lang/*.json, t("schlüssel"))
lang/
  de.json  en.json   Sprach-Strings (ein Platzhalter-Schlüssel je Text)
games/
  snake.py  pong.py  tictactoe.py  breakout.py  tetris.py  invaders.py  game2048.py
```

Die gewählte Sprache wird in `mem.json` gespeichert (im Abschnitt `mem`, neben
dem Abschnitt `highscores` in derselben Datei) und beim nächsten Start
automatisch geladen.

## Plattformhinweise

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

## Installations-Guide

Voraussetzung: **Python 3.9+** (empfohlen 3.12 oder 3.13) und **pygame ≥ 2.6**.

### Windows (empfohlen: automatisch)

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

### Windows / Linux / macOS (manuell)

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

### Linux / macOS mit start.sh

```bash
# Python + venv wie oben (Schritte 2 und 3) einrichten, dann:
chmod +x start.sh      # einmalig, falls noch nicht ausführbar
./start.sh
```

Unter Linux installiert man Python bei Bedarf über den Paketmanager, z. B.
`sudo apt install python3 python3-venv python3-pip` (Debian/Ubuntu), unter macOS
z. B. `brew install python`.

### Andere Python-Version verwenden

`install-python.bat` richtet standardmäßig Python 3.13 ein. Wer 3.12 (oder eine
andere Version) bevorzugt, ändert in der Datei die Zeile `set "PYVER=3.13"` auf die
gewünschte Version und die winget-ID entsprechend (`Python.Python.3.12`).

### Fehlersuche

- **`pygame` nicht gefunden** → venv aktiviert? Schritt 3 wiederholen
  (`pip install -r requirements.txt`).
- **`python` wird nicht erkannt (Windows)** → Python wurde ohne „Add to PATH"
  installiert; neu installieren und Haken setzen, oder `py` statt `python` nutzen.
- **Kein Ton** → in den Optionen „Sound" prüfen; Haptik wirkt nur mit Controller.
- **Fenster/Einbettung unter Linux** → siehe *Plattformhinweise* (Wayland/XWayland).
