# PyGameZ Web – Engine & Portierungs-Leitfaden

Die Web-Version ist reines HTML/CSS/JavaScript **ohne Build-Schritt und ohne Server**:
`index.html` doppelklicken → alles läuft (auch über `file://`). Deshalb gilt:

* **Keine ES-Module** (`import`/`export`), kein `fetch` lokaler Dateien, keine CDNs.
  Alle Dateien sind klassische `<script>`s, die nacheinander geladen werden.
* Jede Spieldatei ist in eine IIFE gekapselt, sonst kollidieren `const`/`class`-Namen
  zwischen den Dateien (alle klassischen Skripte teilen sich einen globalen Scope):

```js
(function () {
  "use strict";
  const { ui, draw, t } = PG;
  // ...
  PG.register(MyGame, { id: "MyGame", key: "my", name: "My Game" });
})();
```

Hilfsdateien, die von einem Spiel geteilt werden (z. B. `cards.js`, `sudoku_gen.js`),
hängen ihre API an ein Namensobjekt, z. B. `PG.cards = {...}` / `PG.sudokuGen = {...}`.

## Aufbau

```
web/
  index.html            Einstieg (Sidebar + Canvas)
  css/style.css
  img/                  Logo
  js/lang/<code>.js     14 Sprachen, 1:1 aus lang/*.json erzeugt (gleiche Keys wie Python!)
  js/wiki/<code>.js     LamaWiki, 1:1 aus lamawiki/*.json
  js/core/util.js       PG.store, PG.t, PG.rand, PG.Rect, PG.settings, PG.highscore, PG.stats
  js/core/ui.js         PG.ui (Themes/Farben/Fonts/Buttons/Panels/Partikel), PG.draw
  js/core/audio.js      PG.audio (synthetisierte Effekte wie audio.py)
  js/core/game.js       PG.Game (Basisklasse), PG.register
  js/core/app.js        Shell: Startbildschirm, Vorspiel-Screen, Loop, Eingabe, Pause, Highscore, Wiki
  js/manifest.js        Reihenfolge der Spiele + Skriptdateien je Spiel
  js/games/*.js         die Spiele
  js/games/wordle_words/<code>.js
                        Wordle-Wortlisten je Sprache, aus woordlistz/ erzeugt;
                        wird erst beim Spielstart nachgeladen (nicht im Manifest)
  tools/                Smoke-Test (headless Chrome) + die Erzeuger für
                        js/lang, js/wiki und js/games/wordle_words
```

## Logische Fläche

Alle Spiele zeichnen auf **800 × 600** logische Pixel (`PG.W`, `PG.H`, bzw. `this.width`,
`this.height`). Die App skaliert das gestochen scharf auf die Fenstergröße (die Canvas hat
intern `PG.app.pixelScale` echte Pixel pro logischem Pixel).

* `draw(ctx)` bekommt den Context bereits mit der Basis-Transformation. **Nie**
  `ctx.setTransform(1,0,0,1,0,0)` / `ctx.resetTransform()` benutzen – stattdessen
  `ctx.save()` … `ctx.restore()`. (Notfalls stellt `PG.app.resetTransform(ctx)` die Basis wieder her.)
* Alles, was man an `globalAlpha`, `globalCompositeOperation`, `clip`, `filter`,
  `shadowBlur`, `lineDash` ändert, mit `save/restore` klammern.
* Pixel-Renderer (Raycaster, 3D, Pixel-Effekte): in eine kleine Offscreen-Canvas
  (`ui.makeCanvas(w, h)`) per `ImageData` rendern und mit `ctx.drawImage(off, 0, 0, W, H)`
  hochskalieren. Statische, teure Ebenen (Bretter, Tische, Hintergründe) einmal in eine
  Offscreen-Canvas zeichnen und cachen (für Schärfe: Größe × `PG.app.pixelScale`, mit
  `drawImage(c, x, y, w, h)` zeichnen).

## Die Spielklasse

```js
class SnakeGame extends PG.Game {
  init()  { /* einmalig vor dem ersten reset() */ }
  reset() { this.score = 0; this.gameOver = false; /* ... */ }
  update(dt) { /* dt in Sekunden, max. 0.1 – wird nicht aufgerufen, wenn paused/gameOver */ }
  draw(ctx)  { /* jedes Frame */ }
  handleEvent(ev) { /* Eingaben */ }
  destroy() { /* optional: beim Verlassen */ }
}
PG.register(SnakeGame, {
  id: "SnakeGame",            // = Python-Klassenname (Akzentfarbe, Wiki-Seite, Manifest)
  key: "snake",               // = highscore_key aus Python
  name: "Snake",              // oder {default: "Chess", de: "Schach", fr: "Échecs", ...} (LocalizedName)
  modes: [["classic", "snake.mode.classic"], ...],  // optional = Python MODES (nur Einzelspieler-Modi!)
  settingsKey: "snake",       // optional: Abschnitt der Spiel-Einstellungen
  defaults: { wrap: false },  // optional: Standardwerte dieses Abschnitts
  wantsRightClick: true,      // optional
});
```

**Wichtig:** Keine Klassenfelder (`foo = 1;` im Klassenrumpf) für Spielzustand – die würden
erst *nach* dem `reset()` im Basiskonstruktor gesetzt und den Zustand überschreiben.
Zustand gehört in `init()` bzw. `reset()`. Getter/Methoden sind unproblematisch.

Vom Basiskonstruktor gesetzt: `width, height, mode, settings (global), opts (Spiel-Abschnitt),
score, gameOver, paused, font (22px), bigFont (48px bold), accent ([r,g,b]), highscoreKey, meta`.

| Python (`game_base.Game`)                  | JavaScript (`PG.Game`)                              |
|-------------------------------------------|-----------------------------------------------------|
| `self.game_over`, `self.paused`, `self.score` | `this.gameOver`, `this.paused`, `this.score`      |
| `handle_event(event)`                     | `handleEvent(ev)`                                   |
| `self.surface` / `draw()`                 | `draw(ctx)` (Context als Parameter)                 |
| `self.is_action(key, "up")`               | `this.isAction(ev.key, "up")` (WASD/Leertaste **und** Pfeile/Enter) |
| `self.play_sound("eat")`                  | `this.playSound("eat")`                             |
| `audio.tone(f, dur, settings, wave, vol)` | `this.tone(f, dur, wave, vol)`                      |
| `self.rumble(ms)`                         | `this.rumble(ms)`                                   |
| `self.report_result(won)` / `self.ach_event(id, v)` | gleich: `reportResult(won)`, `achEvent(id, v)` |
| `self.settings["snake"]["wrap"]`          | `this.opts.wrap` (mit `settingsKey`/`defaults`)     |
| `settings_mod.save_settings(self.settings)` | `this.saveSettings()`                             |
| `highscore.load_highscores().get(key, 0)` | `this.highscore` bzw. `PG.highscore.get(key)`       |
| `wants_right_click`, `wants_escape`, `capture_mouse` | `get wantsRightClick()`, `get wantsEscape()`, `get captureMouse()` |
| `on_surface_changed()`                    | entfällt (feste logische Größe)                     |

Die App übernimmt wie `main.py`: **ESC = Pause** (außer `wantsEscape` ist true – dann bekommt
das Spiel ESC), Highscore sichern beim Übergang zu `gameOver` und beim Verlassen, Highscore-Banner
unten bei Game Over, Konfetti bei Rekord, Neustart-Erkennung (gameOver → false).
Das Spiel selbst startet also wie in Python mit Enter/Leertaste neu (`reset()`).

## Eingabe-Ereignisse

```js
ev = { kind, key, char, pos, button, delta, rel, repeat }
```

| kind          | Felder                                                               |
|---------------|----------------------------------------------------------------------|
| `"keydown"`   | `key` = Tkinter-keysym (`"Up"`, `"Left"`, `"space"`, `"Return"`, `"BackSpace"`, `"Tab"`, `"a"`, `"A"` (mit Shift), `"1"`, `"minus"`, `"plus"`, `"F1"` …), `char` = getipptes Zeichen, `repeat` |
| `"keyup"`     | `key`                                                                |
| `"mousedown"` | `pos` = `[x, y]` logisch (ganzzahlig), `button` 1 = links, 3 = rechts (nur mit `wantsRightClick`) |
| `"mouseup"`   | `pos`, `button`                                                      |
| `"mousemove"` | `pos`                                                                |
| `"wheel"`     | `pos`, `delta` (+1 = hoch, −1 = runter)                              |
| `"mouserel"`  | `rel` = `[dx, dy]` – nur bei Pointer-Lock (`captureMouse` true)       |

Bei `captureMouse === true` fordert die App beim nächsten Klick Pointer-Lock an (dieser Klick
wird nicht ans Spiel weitergereicht), zeigt bis dahin einen Hinweis und pausiert, wenn der Browser
die Maus freigibt (ESC). Touch wird automatisch als Maus geliefert.

## Zeichnen (`PG.ui`, `PG.draw`)

Farben sind wie in Python `[r, g, b]` oder `[r, g, b, a]` (a = 0…255) – oder CSS-Strings.
Theme-Farben **immer dynamisch** lesen: `ui.TEXT`, `ui.TEXT_DIM`, `ui.TEXT_FAINT`, `ui.PANEL`,
`ui.PANEL_LIGHT`, `ui.BORDER`, `ui.BORDER_LIGHT`, `ui.BTN`, `ui.BTN_SEL`, `ui.ACCENT`, `ui.ACCENT2`,
`ui.ACCENT_SOFT`, `ui.GREEN`, `ui.GOLD`, `ui.RED`, `ui.BG_TOP`, `ui.BG_BOTTOM`.

```js
ui.font(px, bold=false, mono=false)   // gecacht; f.size(text) -> [w,h], f.width(text), f.height, f.px
ui.text(ctx, str, x, y, font, color, anchor="topleft", alpha?)  // wie blit(img, img.get_rect(<anchor>=(x,y)))
                                       // anchor: topleft|center|midtop|midbottom|midleft|midright|topright|bottomleft|bottomright
                                       // gibt PG.Rect zurück
ui.gradText(ctx, str, x, y, font, top, bottom, anchor)
ui.wrap(str, font, maxWidth)          // -> Zeilen
ui.col(color, alpha01?)               // -> CSS-String
ui.mix(c1, c2, f), ui.pulse(speed=2, lo=.35, hi=1), ui.now() (Sek.), ui.ticks() (ms)
ui.drawBackground(ctx, w, h, stars=true)
ui.drawPanel(ctx, rect, {color, border, radius, shadow=true, accentTop})
ui.drawButton(ctx, rect, label, font, selected, {accent, sub, subFont})
ui.drawTitle(ctx, width, title, {subtitle, y=52, big, small, accent})  // -> y der Akzentlinie
ui.drawFooter(ctx, width, height, text, font?)
ui.drawOverlayBox(ctx, w, h, title, color, hint|[lines], {big, small, cy})  // Standard-Endstand-Box
ui.spawnBurst(x, y, color), ui.spawnConfetti(w, h), ui.beginTransition()
ui.gameColor(id), ui.makeCanvas(w, h), ui.roundPath(ctx, x, y, w, h, r)
new ui.TextInput(text, maxlen, charset, placeholder)  // .handle(ev), .draw(ctx, rect, font, focused, invalid), .text

PG.draw.rect(ctx, color, rect, width=0, radius=0)      // width>0: Rahmen innen; radius Zahl oder [tl,tr,br,bl]
PG.draw.circle(ctx, color, [cx,cy], r, width=0)
PG.draw.ellipse(ctx, color, rect, width=0)
PG.draw.line(ctx, color, p1, p2, width=1)
PG.draw.lines(ctx, color, closed, points, width=1)
PG.draw.polygon(ctx, color, points, width=0)
PG.draw.arc(ctx, color, rect, start, stop, width=1)    // Radiant, wie pygame
```

`rect` darf überall `PG.Rect` oder `[x, y, w, h]` sein.

## Werkzeuge (`PG`)

```js
PG.t("snake.score", {score: 12})      // identische Keys wie Python (lang/*.json); {x:,} und {x:.1f} gehen
PG.addStrings({de: {...}, en: {...}})  // nur für Web-eigene Texte (Key-Präfix "web.<spiel>.")
PG.pick({default: "..", de: ".."})    // Text je Sprache
PG.rand.random() / uniform(a,b) / randint(a,b) (inklusive) / randrange / choice / shuffle (in place) / sample / weighted / gauss
new PG.Random(seed)                   // deterministisch (Werte weichen von Python ab)
new PG.Rect(x,y,w,h)                  // x,y,w,h,left,right,top,bottom,centerx,centery,center,midtop,...,
                                      // collidepoint(x,y|[x,y]), colliderect, contains, inflate, inflateIp,
                                      // move, moveIp, union, clamp, copy
PG.clamp, PG.lerp, PG.dist, PG.sign, PG.mod (Python-Modulo!), PG.radians, PG.degrees, PG.TAU
PG.store.get(key, def) / PG.store.set(key, val)    // localStorage (JSON)
PG.highscore.get(key) / update(key, score)
PG.audio.play(name) / tone(freq, dur, wave, vol)
PG.downloadText(filename, text) / PG.pickTextFile(accept, (text, name) => ...)   // Export/Import
```

Sound-Namen: `click select eat bounce point shoot explode hit rotate lock line merge move gameover win powerup level`.

## Regeln für die Portierung

1. **Originalgetreu**: Spielregeln, Physik, KI, Level, Punkte, Optik, Texte und In-Game-Optionen
   wie in der Python-Datei. Nur Plattform-Bits werden ersetzt (pygame → Canvas).
2. **Nur Einzelspieler**: Mehrspieler-Modi (2 Spieler an einer Tastatur, "duel", Hot-Seat)
   entfallen komplett. KI-Gegner bleiben. `modes` enthält nur Einzelspieler-Modi.
3. **Replays** (replay.py) entfallen.
4. **Texte** über `PG.t` mit den Python-Keys. Keine Keys erfinden, die es nicht gibt – für neue
   Texte `PG.addStrings` (mindestens de + en).
5. **Nur die eigenen Dateien** anlegen/ändern (siehe Manifest). Keine Änderungen an `core/`,
   `manifest.js`, `index.html`, `css/`.
6. Performance: 60 FPS auf normalen Rechnern. Keine Fonts/Gradients/Canvas pro Objekt pro Frame neu
   erzeugen, wenn es sich vermeiden lässt.

## Smoke-Test

```
node web/tools/smoke.js Game2048 SnakeGame           # Zufallseingaben, meldet Laufzeitfehler
node web/tools/smoke.js Game2048 --shot              # zusätzlich Screenshot -> web/tools/shots/<id>-<mode>.png
node web/tools/smoke.js Game2048 --shot --idle --frames 90    # Startbild ohne Eingaben
node web/tools/smoke.js Game2048 --keys "Right,Right,Up@30,space" --shot   # geskriptete Tasten (@n = Frames warten)
node web/tools/smoke.js all
```
