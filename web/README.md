# PyGameZ Web

Die komplette PyGameZ-Spielesammlung im Browser, als **Einzelspieler-Version**.

## Starten

**`index.html` doppelklicken.** Das war's.

Es braucht keinen Server, keine Installation, kein Internet und keinen Build-Schritt. Die Seite läuft
direkt von der Festplatte (`file://`) in jedem aktuellen Browser (Chrome, Edge, Firefox, Safari).
Man kann den Ordner `web/` auch 1:1 auf einen Webspace (z. B. GitHub Pages) hochladen.

## Was drin ist

* alle Spiele der Desktop-Version (Snake, Tetris, Schach, Minigolf, Tower Defense, Block Jump, …),
  mit derselben Spiellogik, KI, denselben Leveln und Optionen
* Startbildschirm, Vorspiel-Screen und Pause wie in der Desktop-Version
* Highscores, Einstellungen und Statistik werden im Browser gespeichert (localStorage)
* 14 Sprachen (identisch zu `lang/`), LamaWiki als Hilfe (identisch zu `lamawiki/`)
* alle 10 UI-Designs (UI v4.2 Midnight Glass (Standard), UI v4.1, v4.1.1–v4.1.4, UI v4, UI v3 Classic, UI v2, UI v1)
* synthetisierte Soundeffekte wie in der Desktop-Version (keine Audiodateien)
* Minigolf: eigene Bahnen bauen, als `.lamapgzmap` exportieren und importieren
* **Replays** für Minigolf, Bowling, Billard, Pinball, Snake und Tetris: am Rundenende zeigt **P**
  die Wiederholung, **S** legt sie ins Archiv (Sidebar-Knopf **Replays**), **E** teilt sie als
  `.lamapgzreplay`-Datei und **I** liest so eine Datei wieder ein. Die Dateien sind mit der
  Desktop-Version austauschbar - eine dort aufgenommene Runde läuft hier und umgekehrt.

## Unterschiede zur Desktop-Version

* **Nur Einzelspieler**: keine Modi, in denen zwei Spieler an einer Tastatur gegeneinander spielen.
  KI-Gegner bleiben natürlich.
* Speicherort: der Browser statt `mem.json`/`settings.json` (Speicher pro Browser; "Highscores löschen"
  in den Einstellungen setzt sie zurück)
* Replays liegen im localStorage statt in `replay.json`; dort ist der Platz knapp, deshalb passen
  **5** Aufnahmen je Spiel ins Archiv (Desktop: 20) und eine Aufnahme endet nach 10 Minuten.
  Zum Aufheben teilt man sie als Datei.
* Maus-Steuerung für 3D-Spiele (Block Jump, Labyrinth, Aim Trainer): einmal ins Bild klicken,
  dann wird die Maus eingefangen. ESC gibt sie wieder frei und pausiert.

## Steuerung

| Taste          | Wirkung                                  |
|----------------|------------------------------------------|
| WASD / Pfeile  | bewegen (je nach Spiel)                  |
| Leertaste/Enter| Aktion / Neustart nach Game Over         |
| ESC            | Pause / weiter (in Menüs: zurück)        |
| F11            | Vollbild                                 |

## Für Entwickler

Wie die Engine aufgebaut ist und wie man ein Spiel portiert, steht in [`ENGINE.md`](ENGINE.md).
Automatischer Test aller Spiele (headless Chrome):

```
node web/tools/smoke.js all
node web/tools/replay_check.js
```

`replay_check.js` spielt je Spiel mit Aufzeichnung eine echte Partie, fährt die Aufnahme vorwärts,
rückwärts und in Zufallssprüngen durch, vergleicht den Endstand mit dem Original und prüft Teilen,
Einlesen und den Replay-Screen - dazu eingefrorene Aufnahmen der Desktop-Version
(`tools/desktop_replays.js`), damit das Dateiformat beider Fassungen gleich bleibt.

Sprach- und Wiki-Dateien (`js/lang/`, `js/wiki/`) werden aus `lang/*.json` und `lamawiki/*.json`
erzeugt. Nach Änderungen dort neu erzeugen:

```
node web/tools/build-i18n.js
```

Die Wordle-Wortlisten (`js/games/wordle_words/<code>.js`) entstehen genauso aus dem Ordner
`woordlistz/` der Desktop-Version. Nach Änderungen dort neu erzeugen:

```
node web/tools/build-wordlists.js
```
