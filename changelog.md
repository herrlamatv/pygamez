# Changelog

**🌐 Sprache / Language:** **🇩🇪 [Deutsch](#-deutsch)** · **🇬🇧 [English](#-english)**

---

<a name="-deutsch"></a>

## 🇩🇪 Deutsch

### Minigolf-Tour: 360 Bahnen – 2026-08-26

Minigolf wächst vom Achtzehn-Bahnen-Platz zur **Tour**: zu den 18 handgebauten
Bahnen kommen **342 erzeugte** dazu - zusammen **360 Bahnen in 40 Kursen**.

#### Neu
- **Bahn-Generator** `games/minigolf_gen.py`: 38 Tour-Kurse zu je 9 Bahnen,
  vollständig aus einem Seed erzeugt. Kurs 7, Bahn 3 sieht bei jedem Start und
  auf jedem Rechner gleich aus - gespeichert werden muss dafür nichts.
- **Zehn Bahnfamilien** über vier Schwierigkeitsstufen: gerade Bahn, Wandreihen
  mit versetzten Lücken, Dogleg, Gummipuffer-Feld, Wasserteiche, Rampe,
  Chicane, Windmühlen-Korridor, Inselgrün und Wanderblock-Schleuse. Par 2 bis 5,
  Gesamt-Par je Kurs zwischen 26 und 39.
- **Passierbarkeit ist eingebaut, nicht erhofft**: jede Familie legt zuerst den
  Weg vom Abschlag zum Loch fest und baut die Hindernisse darum herum -
  Wandlücken sind nie schmaler als 12 Einheiten (Ball-Durchmesser 3,4), Wasser
  liegt nur neben dem Weg, ein Wanderblock ist stets schmaler als seine Lücke,
  und Mühlenflügel lassen seitlich Platz.
- **Kurswahl im Setup**: vier Knöpfe (Classic / Pro / Tour / Random) und darunter
  eine Zeile mit Pfeilen zum Blättern durch die 38 Tour-Kurse samt Anzeige des
  Gesamt-Pars. Der Bestwert wird je Tour-Kurs einzeln gespeichert.
- **Random** zieht jetzt aus allen 360 Bahnen statt nur aus den 18 gebauten.
- **Aufnehmen abschaltbar**: Die Regel *nach acht Schlägen ist die Bahn vorbei*
  bleibt Standard, lässt sich aber im Setup (oder mit **P**) ausschalten - dann
  wird bis zum Einlochen weitergespielt. Der Setup-Screen hat dafür eine eigene
  AN/AUS-Zeile bekommen und passt seine Höhe jetzt an die Auflösung an, damit
  auch 480x360 alle fünf Blöcke zeigt.
- **Schlag abbrechen mit R**: Wer die Maustaste hält, die Kraft schon geladen
  hat und es sich anders überlegt, drückt **R** - der Ball bleibt liegen, der
  Schlag zählt nicht, und nach dem Loslassen lässt sich ganz normal neu
  aufladen. Ein kurzer Hinweis in der Kopfzeile bestätigt den Abbruch.

#### Geändert
- `tests/newgames_audit.py` prüft nicht mehr nur die 18 gebauten Bahnen, sondern
  **alle 342 erzeugten** dazu: freie Lage von Abschlag und Loch, Einlochbarkeit
  per Solver (simuliert echte Schläge) und Reproduzierbarkeit aus dem Seed.
  Laufzeit rund eine Minute.
- `settings.json` merkt sich den zuletzt gewählten Tour-Kurs (`minigolf.tour`);
  die Platzmaße stehen jetzt einmalig in `minigolf_gen.py`.
- Sidebar-Untertitel und beide READMEs nennen die neue Zahl: 360 Bahnen.

### Minigolf, Pinball & Bowling – 2026-08-26

Drei neue Sportspiele auf einen Schlag (Nr. 40-42): **Minigolf** mit 18
handgebauten Bahnen, ein vollwertiger **Pinball**-Automat mit Multiball und
**Bowling** mit echter Pin-Physik - alle drei mit Mehrspieler-Modus, Erfolgen,
Wiki-Seite und allen 14 Sprachen.

#### Neu
- **Minigolf**: 18 handgebaute Bahnen in drei Kursen - *Classic* (sanfter
  Einstieg), *Pro* (Inselgrün, Doppelmühle, Wanderblöcke) und *Random* (neun
  zufällig gezogene und gespiegelte Bahnen). Sand bremst, Rampen beschleunigen,
  Wasser kostet einen Strafschlag, dazu Gummipuffer, Windmühlen und
  Wanderblöcke. Gezielt wird mit der Maus, die Kraft lädt bei gedrückter
  Maustaste (alternativ Pfeile + Leertaste), **G** blendet die Ziellinie um.
  Scorekarte mit Par je Bahn, **500 Punkte extra für ein Hole-in-One**; zu zweit
  spielt jeder dieselbe Bahn nacheinander. Bestwert je Kurs in `mem.json`
  (Abschnitt `minigolf`).
- **Pinball**: drei Tische - *Classic* (drei Pop-Bumper, eine Target-Bank),
  *Space* (vier Bumper im Karo, zwei Banks) und *Lama* (offenes Feld, sechs
  Targets im Bogen). Mit Slingshots, vier **L-A-M-A**-Rollover-Bahnen, Saucer
  mit Ball-Lock und **Multiball samt Jackpot**, dazu sechs Sekunden Ball-Save,
  Nudge über die Hoch-Taste, **TILT** bei dreimal zu hastigem Anstoßen und ein
  Multiplikator bis x5. 3 oder 5 Bälle je Partie, zu zweit im Wechsel. Ein zu
  schwacher Schuss ist kein Beinbruch: der Ball rollt in die Schussbahn zurück
  und darf noch einmal. Bestwert je Tisch in `mem.json` (Abschnitt `pinball`).
- **Bowling**: zehn Frames mit vollständiger Strike-/Spare-Wertung inklusive
  Bonuswürfen im zehnten Frame (Maximum: die 300) und einer Scorecard mit X, /
  und laufender Summe. Der Wurf läuft in vier Schritten - Position, Ziel, Effet
  und Kraft - über Regler, die von allein pendeln und sich mit Links/Rechts auch
  von Hand einstellen lassen. Zehn Pins mit echter Masse werfen sich gegenseitig
  um; die Bahn ist vorn geölt, der **Hook** greift erst im hinteren Drittel.
  Bestwert je Schwierigkeit in `mem.json` (Abschnitt `bowling`).
- **9 neue Erfolge**: Punkte-Meilensteine für alle drei Spiele plus
  Hole-in-One, Runde unter Par, Multiball, 50 000 Punkte im Pinball, Turkey
  (drei Strikes in Folge) und ein 200er-Bowlingspiel - jetzt **83 insgesamt**.
- **93 Übersetzungs-Keys je Sprache** und **drei neue Wiki-Seiten** in allen
  14 Sprachen (LamaWiki: 43 Seiten); beide READMEs ergänzt.
- **Headless-Audit** `tests/newgames_audit.py`: prüft alle 18 Minigolf-Bahnen
  per Solver auf Einlochbarkeit (und freie Abschlag-/Lochpositionen), jeden
  Pinball-Tisch auf gelungenen Abschuss und ein hängerfreies Partie-Ende sowie
  die Bowling-Wertung gegen Referenzspiele (300er, lauter Spares, gemischt)
  samt Pin-Physik und Frame-Logik.

#### Geändert
- Sidebar: drei neue Piktogramme (Fahne im Loch, Flipper mit Kugel, Pin mit
  Kugel) mit eigenen Akzentfarben; die Sammlung zählt jetzt **42 Spiele**.
- `settings.json` kennt die Abschnitte `minigolf`, `pinball` und `bowling`
  (Kurs & Ziellinie, Tisch & Ballzahl, Schwierigkeit & Zielhilfe) - alle drei
  merken sich ihre Setup-Auswahl.

### Block Jump: Minecraft-Skin – 2026-08-12

Optik-Update: **Block Jump** sieht jetzt aus wie sein Vorbild - echte
Pixeltexturen auf allen Blöcken, eine Steve-Figur, die Hand im Ego-Modus,
ein Beacon-Strahl am Ziel, Pixel-Wolken und ein HUD mit Herzen.

#### Neu
- **Blocktexturen** (16×16, im Code prozedural erzeugt - keine Bilddateien):
  Gras mit grünem Überhang an den Seiten, Erde, Stein, Eichenbretter,
  Diamantblock (Ziel), Schleimblock (der frühere Sprungblock) sowie Holz für
  Leiter und Zaun. Der Software-Renderer zerlegt jede Fläche perspektivisch
  korrekt in Texel-Vierecke, die Flächenhelligkeit folgt dem Vorbild
  (oben hell, Seiten abgestuft, Unterseite dunkel).
- **Detailstufe nach Abstand** mit **Taste T**: hoch / niedrig / aus. Gleich
  eingefärbte Texel werden vorab zu Rechtecken zusammengefasst, dadurch kostet
  die neue Optik statt rund 18.000 nur noch etwa 3.000 Vierecke je Bild
  (1280×720: ~14 ms in *hoch*, ~6 ms in *niedrig*, ~3 ms mit *aus*). Die Wahl
  wird in `settings.json` gespeichert (`blockjump.textures`).
- **Steve als Spielfigur** in der Verfolgerkamera: Kopf mit Gesicht, Torso,
  Arme und Beine schwingen beim Laufen, der Kopf neigt sich mit dem Blick.
- **Hand im Ego-Modus** samt Lauf-Bob, **Beacon-Strahl** über dem Ziel und
  rotierende **Gold-Barren** anstelle der bisherigen Kristall-Coins.
- **Himmel**: Minecraft-Blau mit quadratischer **Sonne** und driftenden
  **Pixel-Wolken** weit über der Karte.
- **HUD im Spielstil**: **Herzen** für die Leben, Gold-Barren als Coin-Zähler,
  Schattenschrift und ein Fadenkreuz wie im Vorbild.

#### Geändert
- Der Sprungblock ist jetzt optisch ein **Schleimblock**, das Ziel ein
  **Diamantblock** mit Lichtstrahl. Spielverhalten, Level-Generierung und
  Physik bleiben unverändert - `tests/blockjump_audit.py` läuft weiterhin
  fehlerfrei durch alle 45 Level.

### Block Jump Bugfixes – 2026-08-12

Wartungs-Update: **Block Jump** ist jetzt tatsächlich durchspielbar - der
Leiter-Aufstieg endete bisher in praktisch jedem Level in einer Sackgasse.

#### Behoben
- **Leitern waren Sackgassen**: Das Folge-Pad wurde direkt über dem
  Leiterschacht gebaut und überschrieb die oberste Sprosse - der Spieler
  stieß beim Klettern mit dem Kopf an und kam nie oben an. Das Pad beginnt
  jetzt hinter der Stützwand, der Schacht bleibt offen (betraf 44 von 45
  geprüften Leveln, inklusive Level 1 aller drei Modi).
- **Leiter-Coin unerreichbar**: Der Coin hing einen Block vor der Leiter
  statt in der Kletterspalte - jetzt wird er beim Aufstieg eingesammelt.
- **Taste C (Maus fangen/frei) war wirkungslos**: Die Zeichenroutine
  überschrieb den Schalter in jedem Frame; der HUD-Hinweis stimmt jetzt
  wieder.
- **Unsichtbare Wände**: Zäune und Sprungblöcke kollidierten als volle
  1×1×1-Blöcke, obwohl sie viel kleiner gezeichnet werden. Zäune sind jetzt
  0,8 Blöcke hoch (bequem überspringbar, wie im README beschrieben),
  Sprungblöcke blockieren seitlich gar nicht mehr und katapultieren auch
  beim Hineinlaufen; die Pad-Fläche unter beiden wird nicht mehr weggecullt
  (sichtbare Löcher) und die 3rd-Person-Kamera bleibt nicht mehr an Zäunen
  hängen.
- **Schwer-Modus**: Bei maximaler Lücke (4 Blöcke) konnten 1 Block tiefe
  Ziel-Pads plus Aufwärtsversatz framegenau-unmögliche Sprünge erzeugen -
  große Lücken erzwingen jetzt ebene/abwärts führende, tiefe Lande-Pads.
- **Tastenbelegung**: Block Jump ignorierte die in den Optionen belegten
  Tasten (fest WASD/Pfeile) - läuft jetzt über die zentrale Belegung wie
  die übrigen Spiele (Pfeiltasten bleiben als Fallback erhalten).
- **Einstellungen gingen verloren**: Ansicht/Motion-Blur/Empfindlichkeit/
  Maus-Richtung überlebten den Neustart nicht (fehlender
  `blockjump`-Abschnitt in den Settings-Defaults).

#### Neu
- **Headless-Audit** `tests/blockjump_audit.py`: prüft alle 45 Level
  (1-15 × 3 Modi) automatisch auf offene Leiterschächte, erreichbare Coins,
  per Sprungphysik schaffbare Lücken und simuliert jeden Leiter-Aufstieg.

### Tower Defense – 2026-08-06

Spiel Nr. 39: ein komplettes **Tower Defense** mit endlosen Wellen, 4 Karten
und 3 Umfangs-Modi - inklusive Erfolgen, Wiki-Seite und Übersetzungen in alle
14 Sprachen.

#### Neu
- **Tower Defense** (`games/lamatowerdefense.py`): Gegner laufen in Wellen einen
  Pfad entlang; Türme daneben bauen, ausbauen (bis 3 Stufen) und mit **70%
  Erstattung verkaufen**. Jeder Durchbruch kostet Leben (Bosse 5), bei 0 ist
  Schluss - die Punkte zählen als Highscore.
- **4 Karten** mit eigenem Pfad und HP-Schwierigkeitsfaktor: **Wiese**,
  **Schlucht**, **Kreuzung** (der Pfad kreuzt sich - stark für Türme, dafür
  zähere Gegner) und **Spießrutenlauf** (gebaut wird nur direkt am Pfad).
  Gesperrte Karten schaltet die beste erreichte Welle frei (5/10/15); die
  Bestwelle je Karte wird gespeichert (Abschnitt `lamatowerdef` in `mem.json`).
- **Endlos-Wellen per Formel** statt Wellen-Skript: Budget, Gegner-HP und
  Beute skalieren mit der Wellen-Nummer (ab Welle 25 zusätzlich exponentiell);
  alle **8 Wellen ein Boss**, jede 5. Welle eine Themen-Welle, der
  Karten-Faktor greift über eine Anlaufkurve erst ab Welle 10 voll.
- **3 Modi** im Vorspiel-Screen: **Klassisch** (7 Türme, Hauptmodus),
  **Kompakt** (4 Türme, 2 Stufen, mehr Startgold) und **Maximal** (11 Türme,
  **A/B-Spezialisierung** auf höchster Stufe, Spezialgegner und
  Aktiv-Fähigkeiten **Meteor [Q] / Frostnova [W] / Goldsegen [E]**).
- **11 Turmtypen**: Pfeil, Kanone (Splash), Frost (bremst + enttarnt),
  Scharfschütze (Riesenreichweite, Panzerbrecher, Luft), Gift (Schaden über
  Zeit), Tesla (Kettenblitz), Banner (+20%-Aura), Mörser (Bogenschuss über
  fast das ganze Feld), Flak (Luftabwehr), Laser (Dauerstrahl mit Aufladung)
  und Goldbank (Einkommen) - jeder mit eigener A/B-Verzweigung im
  Maximal-Modus (22 Zweige, z. B. Doppelschuss/Durchschlag beim Pfeilturm).
- **11 Gegnertypen**: Läufer, Sprinter, Schwarm, Panzer, Gepanzerte
  (Schadensabzug), Regenerierer, Teiler, **Flieger** (eigene Luftroute, nur
  Luftabwehr trifft), **Getarnte** (nur bei Frost/Banner/Scharfschütze
  sichtbar), **Heiler** (Heil-Aura) und **Boss**.
- **Komfort**: Geist-Vorschau mit Reichweitenkreis, Rechtsklick bricht ab,
  **1-9** Schnellwahl, **[F]** Tempo x2, **[G]** alle Reichweiten, Mausrad
  scrollt das Baumenü, **+5% Zinsen** je Bauphase, Wellen-Bonus.
- **5 neue Erfolge**: Punkte-Meilenstein 10 000 sowie **Königsjäger** (erster
  Boss), **Wellenbrecher** (Welle 20), **Festung** (Welle 10 ohne Verlust)
  und **Endausbau** (Turm voll ausgebaut) - jetzt 74 Erfolge insgesamt.
- **76 neue Übersetzungs-Keys je Sprache** und eine **neue LamaWiki-Seite** in
  allen 14 Sprachen (das Wiki hat jetzt 40 Seiten); beide READMEs (DE/EN +
  12 weitere Sprachen) um Tabellenzeile und Feature-Details ergänzt.

#### Geändert
- Sidebar: neues gezeichnetes **Burgturm-Icon** und Terracotta-Akzentfarbe
  für Tower Defense (`ui.GAME_COLORS`); die Spieleliste zählt damit 39 Spiele.

### Erfolge & Statistiken – 2026-07-26

Das Progression-Update: Die Sammlung zählt jetzt, was du spielst - **69 Erfolge**
und dauerhafte **Spielerstatistiken** über alle 38 Spiele, erreichbar über den
neuen Sidebar-Button **„Erfolge & Statistik"** (Pokal-Symbol).

#### Neu
- **69 Erfolge** in drei Kategorien:
  - **Allgemein** (21): sammlungsweite Ziele wie gestartete Partien (1/10/50/200),
    ausprobierte Spiele (5/15/alle 38), Gesamtspielzeit (1 h/5 h/20 h), gebrochene
    Rekorde (1/10/25), Siege (10/50, in 5 verschiedenen Spielen) - plus Extras wie
    **Nachteule** (nach Mitternacht), **Frühaufsteher**, **Polyglott**
    (Sprachwechsel), **Leseratte** (Wiki geöffnet) und **Ganz mein Stil**
    (Snake-Personalisierung).
  - **Punkte-Meilensteine** (30): eine faire Ziel-Punktzahl je Spiel, abgestimmt
    auf dessen Punkteskala (z. B. Snake 500, Tetris 10 000, Flappy 25, Simon 12).
  - **Besondere Momente** (18): Schachmatt gegen die KI, KNIFFEL, die
    2048er-Kachel, Tetris-Vierfachreihe, Blackjack mit zwei Karten, Wordle in
    ≤ 2 Versuchen, Galgenmännchen ohne Fehlversuch, fehlerfreies Sudoku,
    Minesweeper- und Solitär-Sieg, perfektes Memory, leergefressenes
    Pacman-Labyrinth, alle Breakout-Level, alle Frogger-Buchten, 100 %
    Aim-Trainer-Genauigkeit, 2 000 Poker-Chips, Snake-Prestige und
    Competitive-Level 5.
- **Toast-Einblendung**: goldenes Banner mit gezeichnetem Abzeichen und kleinem
  Arpeggio oben rechts beim Freischalten - auch mitten im Spiel.
- **Erfolge-&-Statistik-Screen**: Reiter **Erfolge** mit Gesamtfortschritts-Balken,
  Kategorien, Fortschrittsanzeige je Erfolg und Freischalt-Datum; Reiter
  **Statistiken** mit Übersichtskarten (Gesamtspielzeit, Partien, Siege, Rekorde,
  ausprobierte Spiele, Erfolgs-Stand), **Lieblingsspiel** und einer nach Spielzeit
  sortierten **Pro-Spiel-Tabelle** (Partien · Zeit · Siege · Bestwert). Ein Klick
  auf eine Zeile springt direkt ins Spiel.
- **Persistente Statistiken** je Spiel: Partien (inkl. Neustarts), **aktive**
  Spielzeit (Pausen/Menüs zählen nicht), Siege/Niederlagen (Spiele mit klarem
  Ausgang, im Einzelspieler), gebrochene Rekorde, zuletzt gespielt - gespeichert
  als Abschnitte `stats`/`achievements` in `mem.json`.
- **Bestand zählt**: Beim ersten Start nach dem Update werden vorhandene
  Highscores automatisch angerechnet (ohne Toast-Feuerwerk).
- **94 neue Übersetzungs-Keys in allen 14 Sprachen** (~1 300 neue Strings) und
  eine **neue LamaWiki-Seite** „Erfolge & Statistiken" - ebenfalls in allen
  14 Sprachen (das Wiki hat jetzt 39 Seiten).

#### Geändert
- `game_base.py`: neue Hooks für alle Spiele - `report_result()` (Sieg/Niederlage,
  zählt einmal pro Partie) und `ach_event()` (besondere Momente); in über
  30 Spiele eingebaut.
- Zentrale Game-Loop (`main.py`): zählt Partie-Starts und -Neustarts sowie
  Spielzeit automatisch; die Rekord-Speicherung prüft jetzt zusätzlich
  Meilenstein-Erfolge. Statistiken werden gedrosselt geschrieben (höchstens
  alle 20 s sowie bei Partie-Start/-Ende und beim Beenden).

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

### Minigolf Tour: 360 holes – 2026-08-26

Minigolf grows from an eighteen-hole course into a **Tour**: on top of the 18
hand-built holes come **342 generated** ones - **360 holes across 40 courses**.

#### New
- **Hole generator** `games/minigolf_gen.py`: 38 tour courses of 9 holes each,
  built entirely from a seed. Course 7, hole 3 looks the same on every start and
  on every machine - nothing has to be stored for it.
- **Ten hole families** across four difficulty tiers: straight hole, wall rows
  with offset gaps, dogleg, bumper field, water ponds, ramp, chicane, windmill
  corridor, island green and moving-block gate. Par 2 to 5, total par per course
  between 26 and 39.
- **Passability is built in, not hoped for**: every family first fixes the path
  from tee to cup and builds the obstacles around it - wall gaps are never
  narrower than 12 units (ball diameter 3.4), water only sits beside the path, a
  moving block is always narrower than its gap, and windmill arms leave room at
  the sides.
- **Course picker in the setup**: four buttons (Classic / Pro / Tour / Random)
  and below them a row with arrows to page through the 38 tour courses, showing
  the total par. The best result is stored per tour course.
- **Random** now draws from all 360 holes instead of just the 18 built ones.
- **Pick-up can be switched off**: the rule *after eight strokes the hole is
  over* stays the default but can be turned off in the setup screen (or with
  **P**) - then you keep putting until the ball drops. The setup screen gained
  its own ON/OFF row for it and now scales its height with the resolution so
  that even 480x360 shows all five blocks.
- **Cancel a shot with R**: hold the mouse button, load the power, change your
  mind - press **R** and the ball stays put, the stroke does not count, and
  after releasing you can charge again as usual. A short note in the header
  confirms the cancellation.

#### Changed
- `tests/newgames_audit.py` no longer checks only the 18 built holes but **all
  342 generated** ones as well: free tee and cup positions, being sinkable
  (verified by a solver that simulates real strokes) and reproducibility from the
  seed. Runtime about one minute.
- `settings.json` remembers the last tour course (`minigolf.tour`); the course
  dimensions now live in exactly one place, `minigolf_gen.py`.
- The subtitle and both READMEs carry the new number: 360 holes.

### Minigolf, Pinball & Bowling – 2026-08-26

Three new sports games in one go (no. 40-42): **Minigolf** with 18 hand-built
holes, a full **Pinball** machine with multiball and **Bowling** with real pin
physics - all three with a multiplayer mode, achievements, a wiki page and all
14 languages.

#### New
- **Minigolf**: 18 hand-built holes across three courses - *Classic* (a gentle
  start), *Pro* (island green, double windmill, moving blocks) and *Random*
  (nine holes drawn and mirrored at random). Sand slows you down, ramps
  accelerate, water costs a penalty stroke, plus rubber bumpers, windmills and
  moving blocks. The mouse aims, holding the left button loads the power
  (arrows + space work too), and **G** toggles the aim line. A scorecard shows
  par per hole, **500 bonus points for a hole in one**; in two-player mode each
  player takes the same hole in turn. Best result per course in `mem.json`
  (section `minigolf`).
- **Pinball**: three tables - *Classic* (three pop bumpers, one target bank),
  *Space* (four bumpers in a diamond, two banks) and *Lama* (open playfield, six
  targets in an arc). With slingshots, four **L-A-M-A** rollover lanes, a saucer
  with ball lock and **multiball including jackpot**, plus a six second ball
  save, nudging via the up key, **TILT** after three hasty nudges and a
  multiplier up to x5. 3 or 5 balls per game, alternating in two-player mode. A
  weak plunge is no disaster: the ball rolls back into the shooter lane and you
  may shoot again. Best result per table in `mem.json` (section `pinball`).
- **Bowling**: ten frames with full strike/spare scoring including the bonus
  balls of the tenth frame (maximum: a 300 game) and a scorecard with X, / and
  the running total. The delivery runs in four steps - position, aim, spin and
  power - through sliders that swing on their own and can also be set by hand
  with left/right. Ten pins with real mass knock each other over; the lane is
  oiled up front, so the **hook** only bites in the last third. Best result per
  difficulty in `mem.json` (section `bowling`).
- **9 new achievements**: score milestones for all three games plus hole in one,
  a round under par, multiball, 50,000 points in pinball, a turkey (three
  strikes in a row) and a 200 bowling game - now **83 in total**.
- **93 translation keys per language** and **three new wiki pages** in all
  14 languages (LamaWiki: 43 pages); both READMEs updated.
- **Headless audit** `tests/newgames_audit.py`: checks all 18 minigolf holes
  with a solver for being sinkable (and for free tee/cup positions), every
  pinball table for a successful launch and a game that ends without hanging,
  and the bowling scoring against reference games (a 300, all spares, mixed)
  including pin physics and frame logic.

#### Changed
- Sidebar: three new pictograms (flag in the hole, flipper with ball, pin with
  ball) with their own accent colours; the collection now counts **42 games**.
- `settings.json` knows the sections `minigolf`, `pinball` and `bowling`
  (course & aim line, table & ball count, difficulty & guide) - all three
  remember their setup choice.

### Block Jump: Minecraft skin – 2026-08-12

Visual update: **Block Jump** now looks like the game it is modelled on - real
pixel textures on every block, a Steve figure, the first-person hand, a beacon
beam at the goal, pixel clouds and a HUD with hearts.

#### New
- **Block textures** (16×16, generated procedurally in code - no image files):
  grass with its green overhang on the sides, dirt, stone, oak planks, a
  diamond block (the goal), a slime block (the former spring block) plus wood
  for ladders and fences. The software renderer splits every face into
  perspective-correct texel quads, and face brightness follows the original
  (bright top, graded sides, dark bottom).
- **Distance-based level of detail** on **key T**: high / low / off. Texels
  that end up the same colour are merged into rectangles up front, so the new
  look costs about 3,000 quads per frame instead of roughly 18,000
  (1280×720: ~14 ms on *high*, ~6 ms on *low*, ~3 ms with *off*). The choice is
  stored in `settings.json` (`blockjump.textures`).
- **Steve as the player** in the chase camera: head with a face, torso, arms
  and legs swinging while walking, the head tilting with your view.
- **First-person hand** with walk bob, a **beacon beam** above the goal and
  spinning **gold ingots** instead of the previous crystal coins.
- **Sky**: Minecraft blue with a square **sun** and drifting **pixel clouds**
  high above the map.
- **HUD in the same style**: **hearts** for lives, a gold ingot as the coin
  counter, drop-shadow text and a crosshair like the original.

#### Changed
- The spring block now looks like a **slime block** and the goal like a
  **diamond block** with a light beam. Gameplay, level generation and physics
  are unchanged - `tests/blockjump_audit.py` still passes all 45 levels.

### Block Jump bugfixes – 2026-08-12

Maintenance update: **Block Jump** is now actually beatable - the ladder
climb used to end in a dead end in practically every level.

#### Fixed
- **Ladders were dead ends**: The follow-up pad was built directly on top of
  the ladder shaft and overwrote the top rung - the player bumped their head
  while climbing and never reached the top. The pad now starts behind the
  support wall, keeping the shaft open (affected 44 of 45 audited levels,
  including level 1 of all three modes).
- **Ladder coin unreachable**: The coin floated one block in front of the
  ladder instead of inside the climbing column - it is now collected during
  the ascent.
- **The C key (capture/release mouse) did nothing**: The draw routine
  overwrote the toggle every frame; the HUD hint is accurate again.
- **Invisible walls**: Fences and spring blocks collided as full 1×1×1
  cubes even though they are drawn much smaller. Fences are now 0.8 blocks
  tall (comfortably jumpable, as the README describes), spring blocks no
  longer block sideways at all and also catapult when walked into; the pad
  surface underneath both is no longer culled away (visible holes) and the
  third-person camera no longer snags on fences.
- **Hard mode**: At the maximum gap (4 blocks), 1-block-deep target pads
  plus an upward offset could produce frame-perfect-impossible jumps -
  large gaps now force level/downhill, deep landing pads.
- **Key bindings**: Block Jump ignored the keys configured in the options
  (hardcoded WASD/arrows) - it now uses the central bindings like the other
  games (arrow keys remain as a fallback).
- **Settings were lost**: View/motion blur/sensitivity/mouse direction did
  not survive a restart (missing `blockjump` section in the settings
  defaults).

#### Added
- **Headless audit** `tests/blockjump_audit.py`: automatically checks all
  45 levels (1-15 × 3 modes) for open ladder shafts, reachable coins, gaps
  clearable by the jump physics, and simulates every ladder climb.

### Tower Defense – 2026-08-06

Game no. 39: a complete **tower defense** with endless waves, 4 maps and
3 content modes - including achievements, a wiki page and translations into
all 14 languages.

#### Added
- **Tower Defense** (`games/lamatowerdefense.py`): enemies march along a path in
  waves; build towers next to it, upgrade them (up to 3 levels) and sell at a
  **70% refund**. Every breakthrough costs lives (bosses 5), at 0 the run
  ends - your points count as the high score.
- **4 maps**, each with its own path and HP difficulty factor: **Meadow**,
  **Canyon**, **Crossroads** (the path crosses itself - strong for towers,
  but tougher enemies) and **Gauntlet** (building only right next to the
  path). Locked maps unlock via your best wave (5/10/15); the best wave per
  map is stored (section `lamatowerdef` in `mem.json`).
- **Endless waves by formula** instead of a wave script: budget, enemy HP and
  bounty scale with the wave number (exponentially after wave 25); a **boss
  every 8 waves**, a themed wave every 5th, and the map factor ramps up fully
  only from wave 10.
- **3 modes** on the pre-game screen: **Classic** (7 towers, the main mode),
  **Compact** (4 towers, 2 levels, more starting gold) and **Maximal**
  (11 towers, **A/B specialisation** at top level, special enemies and active
  abilities **Meteor [Q] / Frost Nova [W] / Gold Rush [E]**).
- **11 tower types**: Arrow, Cannon (splash), Frost (slows + reveals), Sniper
  (huge range, armor piercing, anti-air), Poison (damage over time), Tesla
  (chain lightning), Banner (+20% aura), Mortar (arcing shots across most of
  the field), Flak (anti-air), Laser (ramping beam) and Gold Bank (income) -
  each with its own A/B branch in Maximal mode (22 branches, e.g. Double
  Shot/Piercing on the arrow tower).
- **11 enemy types**: runt, sprinter, swarm, tank, armored (flat damage
  reduction), regenerator, splitter, **flyer** (own air route, only anti-air
  hits), **cloaked** (visible only near Frost/Banner/Sniper), **healer**
  (heal aura) and **boss**.
- **Comfort**: ghost preview with range circle, right-click cancels, **1-9**
  quick select, **[F]** 2x speed, **[G]** all ranges, mouse wheel scrolls the
  build bar, **+5% interest** per build phase, wave bonuses.
- **5 new achievements**: a 10,000-point milestone plus **Boss Hunter**
  (first boss), **Wavebreaker** (wave 20), **Fortress** (wave 10 without
  losses) and **Full Build** (fully upgraded tower) - 74 achievements total.
- **76 new translation keys per language** and a **new LamaWiki page** in all
  14 languages (the wiki now has 40 pages); both READMEs (DE/EN + 12 more
  languages) extended with a table row and feature details.

#### Changed
- Sidebar: new hand-drawn **castle-tower icon** and terracotta accent colour
  for Tower Defense (`ui.GAME_COLORS`); the game list now counts 39 games.

### Achievements & Statistics – 2026-07-26

The progression update: the collection now tracks what you play - **69
achievements** and persistent **player statistics** across all 38 games,
reachable via the new **"Achievements & Stats"** sidebar button (trophy icon).

#### Added
- **69 achievements** in three categories:
  - **General** (21): collection-wide goals such as games started (1/10/50/200),
    games tried (5/15/all 38), total play time (1 h/5 h/20 h), records broken
    (1/10/25), wins (10/50, in 5 different games) - plus extras like **Night
    Owl** (play after midnight), **Early Bird**, **Polyglot** (switch language),
    **Bookworm** (open the wiki) and **My Own Style** (Snake personalisation).
  - **Score milestones** (30): one fair target score per game, tuned to its
    scoring scale (e.g. Snake 500, Tetris 10,000, Flappy 25, Simon 12).
  - **Special moments** (18): checkmating the AI, a YAHTZEE, the 2048 tile, a
    Tetris quadruple line clear, a two-card Blackjack, a Wordle in ≤ 2 tries,
    Hangman without a miss, a flawless Sudoku, Minesweeper and Solitaire wins,
    a perfect Memory run, an emptied Pac-Man maze, all Breakout levels, all
    Frogger bays, 100% Aim Trainer accuracy, 2,000 poker chips, Snake prestige
    and competitive level 5.
- **Toast notification**: a golden banner with a drawn badge and a little
  arpeggio in the top-right corner on unlock - even mid-game.
- **Achievements & statistics screen**: an **Achievements** tab with an overall
  progress bar, categories, per-achievement progress and unlock dates; a
  **Statistics** tab with overview cards (total play time, games, wins,
  records, games tried, achievement count), your **favourite game** and a
  **per-game table** sorted by play time (plays · time · wins · best).
  Clicking a row jumps straight to that game.
- **Persistent statistics** per game: plays (including restarts), **active**
  play time (pauses/menus don't count), wins/losses (games with a clear
  outcome, in single-player), records broken, last played - stored as the
  `stats`/`achievements` sections of `mem.json`.
- **Existing data counts**: on the first launch after the update, your old
  high scores are credited automatically (without the toast fireworks).
- **94 new translation keys in all 14 languages** (~1,300 new strings) and a
  **new LamaWiki page** "Achievements & Statistics" - also in all 14 languages
  (the wiki now has 39 pages).

#### Changed
- `game_base.py`: new hooks for every game - `report_result()` (win/loss,
  counted once per round) and `ach_event()` (special moments); wired into
  30+ games.
- Central game loop (`main.py`): counts game starts/restarts and play time
  automatically; saving a record now also checks milestone achievements.
  Statistics are written throttled (at most every 20 s plus on game
  start/end and on quit).

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
