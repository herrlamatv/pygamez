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

## Unterschiede zur Desktop-Version

* **Nur Einzelspieler**: keine Modi, in denen zwei Spieler an einer Tastatur gegeneinander spielen.
  KI-Gegner bleiben natürlich.
* keine Replays
* Speicherort: der Browser statt `mem.json`/`settings.json` (Speicher pro Browser; "Highscores löschen"
  in den Einstellungen setzt sie zurück)
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
```

Sprach- und Wiki-Dateien (`js/lang/`, `js/wiki/`) werden aus `lang/*.json` und `lamawiki/*.json`
erzeugt. Nach Änderungen dort neu erzeugen:

```
node web/tools/build-i18n.js
```
