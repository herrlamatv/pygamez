# Spielesammlung (Tkinter + Pygame)

Eine Desktop-Spielesammlung in Python: **Tkinter** bildet Fenster und Menue,
**Pygame** wird als Spiel-Display in das Tkinter-Fenster eingebettet.

## Installation & Start

```bash
pip install pygame
# oder:  pip install -r requirements.txt

python main.py
```

Python 3.9+ empfohlen (getestet mit Python 3.12 / pygame 2.6).

## Bedienung

- Spiel links im Menue per Button waehlen. Danach erscheint ein **Vorspiel-Screen**
  (wie bei Minecraft): **Einzelspieler** oder **Mehrspieler** waehlen, zu den
  **Optionen** gehen oder zurueck. Pfeile/Maus zum Waehlen, Enter startet.
- **ESC** = Pause / weiter (in Menues: zurueck).
- **F11** (oder Button "Vollbild an/aus") = Vollbild ein/aus. Das Tkinter-Fenster
  geht in den Vollbildmodus, das Pygame-Display bleibt darin eingebettet ("im
  Fenster") und der Spielinhalt wird seitenverhaeltnistreu hochskaliert
  (schwarze Raender bei abweichendem Seitenverhaeltnis). Fenster kann auch frei
  in der Groesse gezogen werden.
- **"Zurueck zum Menue"**-Button beendet das Spiel und speichert den Highscore.
- **Beenden**-Button schliesst Pygame und Tkinter sauber.

| Spiel        | Steuerung                                  |
|--------------|--------------------------------------------|
| Snake        | Frei belegbar (Standard P1=WASD, P2=Pfeile). **Mehrspieler**: 2 Schlangen |
| Pong         | Frei belegbar (Standard P1=W/S, P2=Pfeile). **Mehrspieler**: P2 statt KI  |
| Tic-Tac-Toe  | Setup: Schwierigkeit (Easy/Medium/Hard) + Brettgroesse 3x3..9x9, dann Maus. Bei Game Over: Enter/Klick = neue Runde, S = Einstellungen |
| Breakout     | Setup: 1/2/3 Schwierigkeit, Links/Rechts Ballfarbe, Hoch/Runter Startlevel (oben rechts), M Aufbau (Standard/Voll), Enter Start. Spiel: Maus/Pfeile, Leertaste startet den Ball |
| Tetris       | Frei belegbar: Links/Rechts verschieben, Hoch drehen, Runter Soft-Drop, Aktion Hard-Drop. **Mehrspieler**: Versus (2 Felder) |
| Invaders     | Frei belegbar: Links/Rechts bewegen, Aktion schiessen. Wellen leeren, Leben schuetzen |
| 2048         | Frei belegbar (Standard Pfeile/WASD) schieben; gleiche Zahlen verschmelzen. Ziel: 2048 |

Highscores werden in `highscores.json` (neben dem Code) gespeichert.

## Optionen, Steuerung & Mehrspieler

Ueber den Button **"Optionen / Steuerung"** (links) oder aus dem Vorspiel-Screen
oeffnet sich der Options-Bildschirm im Spielbereich:

- **Sound** an/aus, **Lautstaerke** und **Haptik** (Gamepad-Vibration, nur mit
  angeschlossenem Controller wirksam) - jeweils per Links/Rechts umschalten.
- **Vorlagen** fuer die Steuerung: *WASD + Pfeile*, *WASD + IJKL*, *Pfeile + WASD*.
- **Jede einzelne Taste** fuer Spieler 1 und Spieler 2 frei belegen: Zeile
  waehlen, Enter druecken, gewuenschte Taste druecken (Esc bricht ab).

Einstellungen werden dauerhaft in `settings.json` gespeichert. Im **Einzelspieler**
steuern beide Belegungen dieselbe Figur (Standard: WASD *und* Pfeile funktionieren),
im **Mehrspieler** je eine. Mehrspieler gibt es fuer **Snake**, **Pong** und
**Tetris (Versus)**.

Alle Spiele haben **Soundeffekte** (prozedural erzeugt, keine Extra-Dateien noetig),
die sich global stummschalten lassen.

## Projektstruktur

```
main.py          Tkinter-Oberflaeche, Pygame-Einbettung, zentrale Game-Loop
game_base.py     Game-Basisklasse (update/draw/handle_event) + InputEvent + Helfer
settings.py      Einstellungen (Sound/Haptik/Tastenbelegung) laden/speichern (JSON)
audio.py         Prozedurale Soundeffekte + Gamepad-Rumble
menu.py          Vorspiel-Screen (Modus) + Options-Screen (Sound/Steuerung)
highscore.py     Laden/Speichern der Highscores (JSON)
games/
  snake.py
  pong.py
  tictactoe.py
  breakout.py
  tetris.py
  invaders.py
  game2048.py
```

## Plattformhinweise

- **Windows**: funktioniert mit pygame 2 (SDL2) direkt. `SDL_VIDEODRIVER` wird
  bewusst NICHT gesetzt (Standardtreiber). Das alte `windib` galt nur fuer
  pygame 1.9 / SDL1 und wuerde unter SDL2 einen Fehler verursachen.
- **Linux/X11**: setzt `SDL_VIDEODRIVER=x11`. Unter **Wayland** klappt die
  Einbettung ueber `SDL_WINDOWID` in der Regel nicht zuverlaessig; dann hilft
  meist XWayland zusammen mit `SDL_VIDEODRIVER=x11`.
- **macOS**: Die Einbettung ueber `SDL_WINDOWID` wird von SDL2 dort nicht
  unterstuetzt; das Programm laeuft, das Pygame-Fenster wird aber nicht in
  Tkinter eingebettet.
