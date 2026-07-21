# Changelog

**🌐 Sprache / Language:** **🇩🇪 [Deutsch](#-deutsch)** · **🇬🇧 [English](#-english)**

---

<a name="-deutsch"></a>

## 🇩🇪 Deutsch

### Sprach-Erweiterung: 9 neue Sprachen – 2026-07-21

Die Oberfläche gibt es jetzt in **14 Sprachen**. Neu hinzugekommen sind neun
Sprachen, die – wie zuvor Spanisch und Portugiesisch – beim ersten Start hinter
dem Knopf **„Weitere Sprachen"** liegen: **Polski, Türkçe, Dansk, Norsk,
Svenska, Suomi, Čeština, Slovenščina, Hrvatski**.

#### Neu
- **9 neue UI-Sprachen**, je vollständig übersetzt (802 Texte pro Sprache):
  Polnisch, Türkisch, Dänisch, Norwegisch, Schwedisch, Finnisch, Tschechisch,
  Slowenisch, Kroatisch.
- **LamaWiki komplett mitübersetzt**: alle 38 Wiki-Seiten in jeder der neun
  neuen Sprachen.
- **Dokumentation**: für jede der neun Sprachen ein vollständiger
  README-Abschnitt in `other.readme.md` (Format wie FR/ES/PT).
- Die neuen Sprachdateien liegen gebündelt in `lang/lang.expansion/` bzw.
  `lamawiki/lang.expansion/` – die Kern-Sprachen (de/en/fr/es/pt) bleiben unberührt.

#### Geändert
- **Sprachauswahl als Raster**: Sprach- und Willkommens-Screen zeigen die nun
  14 Sprachen in einem responsiven Raster mit automatischer Schriftanpassung –
  passt sauber von 480×360 bis 1280×960 (langer Name wie „Slovenščina" läuft
  nicht mehr über).
- Loader (`i18n.py`, `lamawiki.py`) durchsuchen zusätzlich die
  `lang.expansion`-Unterordner; fehlt ein Text, wird wie gehabt auf Deutsch
  zurückgegriffen.
- Build-Skripte (`pyinstall.bat`, `pyinstall-pyarmor.bat`) packen die neuen
  `lamawiki/lang.expansion/*.json` mit in die EXE.

#### Hinweise
- Wordle und Hangman nutzen für die neuen Sprachen vorerst die englischen
  Wortlisten (eigene Listen können später ergänzt werden).

### Games Rework – 2026-07-19

Die bisher größte Überarbeitung: **alle 38 Spiele** wurden in einem Durchgang auf
einen einheitlichen Stand gebracht (Optik, Konsistenz, Übersetzungen, Bugfixes).

#### Neu
- **Einheitlicher Spiel-Look**: Alle 38 Spiele nutzen jetzt die Theme-Palette und
  -Schrift des Menüs. HUDs, Setup-Screens und Overlays folgen dem in den Optionen
  gewählten Design (v4.1 / v4 / Classic) - die Spielfelder behalten ihre
  Identitätsfarben (Filz-Grün, Pacman-Labyrinth, Tetris-Steine …).
- **Sprachabhängige Spielnamen** im Menü: Schach → *Chess/Échecs/Ajedrez/Xadrez*,
  Mühle → *Nine Men's Morris*, Vier gewinnt → *Connect Four*, Panzer-Duell →
  *Tank Duel*, 3D-Labyrinth → *3D Maze*, Dame → *Checkers*, Billard → *Billiards*,
  Galgenmännchen → *Hangman*, Schiebepuzzle → *Sliding Puzzle*.
- **Responsive überall**: Jedes Spiel übernimmt Auflösungswechsel mitten im Spiel
  sauber (Schriften, Layout, Spielfeld) - 11 ältere Spiele konnten das vorher gar
  nicht, 3 weitere nur teilweise.
- **Blackjack**: Austeil- und Aufdeck-Animationen laufen jetzt wirklich (Karten
  fliegen aus dem Schuh, Hole-Card dreht sich mit Sound) - der Code existierte,
  wurde aber nie ausgeführt.
- 13 neue Übersetzungs-Keys in **allen 5 Sprachen** (u. a. Schwierigkeitsgrade in
  Breakout/Tic-Tac-Toe, Flappy-Medaillen, Sudoku-Löschtaste).

#### Geändert
- Gemeinsame Basis (`game_base.py`): sprachabhängige Namen (`LocalizedName`),
  dokumentierte Hooks (`on_surface_changed`, `capture_mouse`, `MODES`),
  Theme-Schriften und Akzentfarbe für alle Spiele automatisch.
- Setup-Screens einheitlich über die UI-Bausteine (`draw_title`, `draw_button`,
  `draw_footer`), Layouts skalieren mit der Fensterhöhe statt fester Pixelwerte.
- Game-Over einheitlich: transluzentes Panel mit Akzent-Rahmen, pulsierender
  Hinweis; **Enter und Leertaste** (oft auch Mausklick) starten überall neu.
- Kartenspiele: gemeinsamer Tisch-Look (`make_felt`), themenfarbene Kartenrücken
  in der Akzentfarbe des jeweiligen Spiels.
- Performance: Vollflächen-Alpha-Fills pro Frame durch gecachte Overlays ersetzt
  (u. a. Aim Trainer, Tunnel Racer, Pacman, Tetris, Tanks, Sudoku, Simon);
  2048 erzeugte pro Kachel und Frame eine neue Schriftart - jetzt gecacht.
- Aufgeräumt: 15 duplizierte Akzentfarb-Konstanten, tote Variablen/Zweige und
  ungenutzte Konstanten entfernt.

#### Behoben
- **Breakout**: Bomben-Steine explodierten doppelt (doppelte Punkte/Drops);
  Feuerball übersprang Steine bzw. traf falsche; Power-Up-Fallgeschwindigkeit
  war framerate-abhängig.
- **Schach**: Maus-Cursor saß gespiegelt, wenn der Mensch Schwarz spielte;
  „Schwarz gewinnt" war im Mehrspieler praktisch unsichtbar (schwarz auf dunkel).
- **Mühle**: Schloss die KI eine Mühle, verschwand die Anzeige sofort - jetzt
  leuchtet sie ~1,2 s golden.
- **Solitär**: Undo nach dem Talon-Ziehen ließ Karten dauerhaft aufgedeckt
  (Schummel-Bug); Undo während des Ziehens konnte Stapel beschädigen.
- **Poker**: Klickflächen zum Halten in 5 Card Draw saßen unterhalb der Karten;
  Pleite im Video Poker kam ohne Sound.
- **Frogger**: Nach Game Over lief die Spiellogik weiter - der Timer „tötete"
  den Frosch in Schleife (Leben negativ, Sound-/Vibrations-Spam).
- **Snake**: Nach einem Auflösungswechsel startete das Brett mit den alten Maßen.
- **Panzer-Duell**: Pfeiltasten waren im Einzelspieler tot; Auflösungswechsel am
  Match-Ende zeichnete die alte Arena.
- **T-Rex Runner**: Umbelegte Sprungtaste blieb beim Loslassen „hängen"
  (Dauer-Niedrigsprung).
- **Minesweeper**: Rechtsklick auf den Smiley startete das Spiel neu;
  Schwierigkeitsname konnte den Minenzähler überlappen.
- **Asteroids**: Schiffe spawnten nach Auflösungswechsel außerhalb des Bildes;
  Steuerungs-Hinweis erschien nie, wenn man länger im Setup war.
- **Pacman**: Punkte-Popups saßen nach Auflösungswechsel an falschen Stellen.
- **Invaders**: Fliegende Schüsse wechselten die Farbe beim Waffenwechsel.
- **Pong**: Namens-Label lief bei langen Übersetzungen aus dem Bild.
- **Doodle Jump**: Blickrichtung des Doodlers wurde nie gespiegelt;
  Monster-Kill mit Propellerhut war stumm.
- **Tic-Tac-Toe**: Schwierigkeits-Buttons waren im Mehrspieler klickbar,
  obwohl ausgegraut (jetzt deaktiviert).
- **Dame**: „S = Setup" funktionierte entgegen dem Hinweis nur im Einzelspieler.
- **Reversi/Vier gewinnt**: Sieger-Text im Mehrspieler war fast unlesbar;
  irreführender „S = Setup"-Hinweis entfernt.
- **Aim Trainer / 3D-Labyrinth / Tunnel Racer**: Auflösungswechsel baute nur
  eine von vier Schriften neu.

---

<a name="-english"></a>

## 🇬🇧 English

### Language expansion: 9 new languages – 2026-07-21

The interface is now available in **14 languages**. Nine new ones were added
which — like Spanish and Portuguese before them — hide behind the **"More
languages"** button on first launch: **Polski, Türkçe, Dansk, Norsk, Svenska,
Suomi, Čeština, Slovenščina, Hrvatski**.

#### New
- **9 new UI languages**, each fully translated (802 strings per language):
  Polish, Turkish, Danish, Norwegian, Swedish, Finnish, Czech, Slovenian,
  Croatian.
- **LamaWiki fully translated too**: all 38 wiki pages in each of the nine new
  languages.
- **Documentation**: a complete README section for every new language in
  `other.readme.md` (same format as FR/ES/PT).
- The new language files are bundled in `lang/lang.expansion/` and
  `lamawiki/lang.expansion/` — the core languages (de/en/fr/es/pt) are untouched.

#### Changed
- **Grid language picker**: the language and welcome screens now lay out the 14
  languages in a responsive grid with automatic font sizing — fits cleanly from
  480×360 to 1280×960 (long names like "Slovenščina" no longer overflow).
- Loaders (`i18n.py`, `lamawiki.py`) also search the `lang.expansion`
  subfolders; a missing string still falls back to German.
- Build scripts (`pyinstall.bat`, `pyinstall-pyarmor.bat`) now bundle the new
  `lamawiki/lang.expansion/*.json` into the EXE.

#### Notes
- Wordle and Hangman use the English word lists for the new languages for now
  (dedicated word lists can be added later).

### Games Rework – 2026-07-19

The biggest overhaul so far: **all 38 games** were brought to a common standard
in one pass (visuals, consistency, translations, bug fixes).

#### Added
- **Unified in-game look**: all 38 games now use the menu's theme palette and
  font. HUDs, setup screens and overlays follow the design chosen in the options
  (v4.1 / v4 / Classic) - playfields keep their identity colours (felt green,
  Pac-Man maze, Tetris pieces …).
- **Language-aware game names** in the menu: Schach → *Chess/Échecs/Ajedrez/
  Xadrez*, Mühle → *Nine Men's Morris*, Vier gewinnt → *Connect Four*,
  Panzer-Duell → *Tank Duel*, 3D-Labyrinth → *3D Maze*, Dame → *Checkers*,
  Billard → *Billiards*, Galgenmännchen → *Hangman*, Schiebepuzzle → *Sliding
  Puzzle*.
- **Responsive everywhere**: every game now cleanly handles mid-game resolution
  changes (fonts, layout, playfield) - 11 older games couldn't do this at all
  before, 3 more only partially.
- **Blackjack**: deal and reveal animations actually play now (cards fly from
  the shoe, the hole card flips with sound) - the code existed but never ran.
- 13 new translation keys in **all 5 languages** (incl. Breakout/Tic-Tac-Toe
  difficulty labels, Flappy medals, Sudoku erase key).

#### Changed
- Shared base (`game_base.py`): language-aware names (`LocalizedName`),
  documented hooks (`on_surface_changed`, `capture_mouse`, `MODES`), themed
  fonts and per-game accent colour provided automatically.
- Setup screens unified on the UI building blocks (`draw_title`, `draw_button`,
  `draw_footer`); layouts scale with window height instead of fixed pixels.
- Unified game over: translucent panel with accent border, pulsing hint;
  **Enter and Space** (often mouse click too) restart everywhere.
- Card games: shared table look (`make_felt`), themed card backs in each game's
  accent colour.
- Performance: per-frame full-surface alpha fills replaced by cached overlays
  (Aim Trainer, Tunnel Racer, Pac-Man, Tetris, Tanks, Sudoku, Simon, and more);
  2048 created a new font per tile per frame - now cached.
- Cleanup: 15 duplicated accent-colour constants, dead variables/branches and
  unused constants removed.

#### Fixed
- **Breakout**: bomb bricks exploded twice (double points/drops); fireball
  skipped bricks or hit wrong ones; power-up fall speed was framerate-dependent.
- **Chess**: mouse cursor was mirrored when the human played Black; "Black wins"
  was virtually invisible in multiplayer (black on dark).
- **Nine Men's Morris**: when the AI closed a mill the highlight vanished
  instantly - it now glows gold for ~1.2 s.
- **Solitaire**: undoing a stock draw left cards permanently face-up (cheat
  bug); undo while dragging could corrupt piles.
- **Poker**: hold click areas in 5 Card Draw sat below the cards; going broke in
  Video Poker was silent.
- **Frogger**: game logic kept running after game over - the timer "killed" the
  frog in a loop (negative lives, sound/rumble spam).
- **Snake**: after a resolution change the board restarted with the old size.
- **Tank Duel**: arrow keys were dead in single-player; resizing on the match
  end screen drew the old arena.
- **T-Rex Runner**: a re-bound jump key got "stuck" on release (permanent low
  jumps).
- **Minesweeper**: right-clicking the smiley restarted the game; the difficulty
  name could overlap the mine counter.
- **Asteroids**: ships respawned off-screen after a resolution change; the
  controls hint never appeared after spending time in setup.
- **Pac-Man**: score popups sat at wrong positions after a resolution change.
- **Invaders**: bullets in flight changed colour when switching weapons.
- **Pong**: name label ran off-screen with long translations.
- **Doodle Jump**: the doodler's snout never mirrored with its direction;
  propeller-hat monster kills were silent.
- **Tic-Tac-Toe**: difficulty buttons were clickable in multiplayer despite
  being greyed out (now disabled).
- **Checkers**: "S = setup" only worked in single-player despite the hint.
- **Reversi/Connect Four**: multiplayer winner text was barely readable;
  misleading "S = setup" hint removed.
- **Aim Trainer / 3D Maze / Tunnel Racer**: resolution changes rebuilt only one
  of four fonts.
