# PyGameZ in Rust

Die komplette Spielesammlung PyGameZ – alle **50 Spiele**, das Programm mit
Seitenleiste, Menü und Vorspiel-Bildschirmen, Profile, Erfolge & Statistik,
Match-Historie, Tages-Archiv, LamaWiki und Replay-Archiv – als natives
Rust-Programm auf [rustgame](../rustgame), dem pixelgenauen Nachbau von
pygame-ce.

Jede Datei ist **Zeile für Zeile aus der Python-Vorlage übertragen**, nicht
nachempfunden: gleiche Konstanten, gleiche Reihenfolge der Zeichenaufrufe,
gleiche Zufallsaufrufe. Deshalb sieht die Rust-Fassung nicht nur *so ähnlich*
aus wie Python, sondern **Pixel für Pixel gleich** – das wird für jedes Spiel
und jeden Bildschirm automatisch gemessen (siehe „Prüfung").

## Bauen und starten

```powershell
$env:PATH = "$env:USERPROFILE\.cargo\bin;$env:PATH"
cd rust
cargo build -p pygamez --release              # beim ersten Mal: lädt die Kisten von crates.io
cargo build -p pygamez --release --offline    # danach geht es auch ohne Netz
..\rust.output\release\pygamez.exe
```

Die Bauausgabe liegt in `rust.output/` (neben `rust/`), das fertige
Programm ist `rust.output/release/pygamez.exe` – eine einzelne Datei mit
dem PyGameZ-Logo als Symbol, ohne Python. Gebraucht wird Rust ab 1.88 mit
der MSVC-Werkzeugkette (unter Windows der Standard).

Spielstände teilt sich die Rust-Fassung mit Python (`mem.json`,
`settings.json`, `stats.pgzlamastats`, `matches.pgzlamamatches`, `ugc.json`,
`replay.json`) – und schreibt sie **Byte für Byte so wie Python**, man kann also jederzeit
zwischen beiden Fassungen wechseln. Mit `PYGAMEZ_DATA_DIR` lässt sich ein
anderer Datenordner wählen.

## Aufbau

| Paket | Inhalt |
|---|---|
| `crates/pgz-core` | Daten ohne Grafik: Speicher, Profile, Sprachen (14), Einstellungen, Highscores, Erfolge (274), Statistik, Match-Historie, Level, Replays, Wortfilter, Lama-Bank, erzeugte Stammdaten (`defs.rs`) |
| `crates/pgz-engine` | Python-Semantik (`py`: Zufall bitgleich zu CPython, `//`/`%`/`round`/`hypot` wie Python), pygame-Fassade (`pg`), `ui.py` (10 Designs), Audio, Erfolgs-Einblendungen, Dateidialoge, Tages-Archiv (Datums-Leiste, Kalender), `Game`-Trait |
| `crates/pgz-games` | Die 50 Spiele, je Datei eine Vorlage aus `games/` (plus Hilfsmodule wie `tetris_core`, `chess_engine`, `geodash_core`, `minigolf_edit`) |
| `crates/pygamez` | Das Programm: Fenster, Seitenleiste, Startbildschirm, Vorspiel/Optionen/Sprache/Willkommen, Profile, Erfolge & Statistik (mit Diagrammen), Match-Historie, LamaWiki, Replay-Archiv und -Player |

`crates/pgz-games/src/lib.rs` wird von `tools/gen_games_list.py` geschrieben
(Reihenfolge wie `games/__init__.py`), `pgz-core/src/defs.rs` von
`tools/export_defs.py`.

## Prüfung

Alle Werkzeuge liegen in `tools/` und laufen aus dem Projektordner mit der
Python-Umgebung des Projekts (`.venv\Scripts\python.exe`).

* **`vergleich.py <spiel>`** – startet dasselbe Spiel in Python und in Rust
  mit gleichem Startwert, gleicher Bild-Uhr, gleichen Eingaben und frischem
  Datenordner und vergleicht das letzte Bild Pixel für Pixel
  (`--mode`, `--frames`, `--keys`/`--keys-file`, `--seed`).
  Mit `screen:<name>[:arg]` statt eines Spiels werden Bildschirme verglichen
  (`menu` = Startbildschirm, `profiles`, `pick:<spiel>`, `levelintro`,
  `progress[:reiter]`, `wiki[:seite]`, `replays`, `pregame:<spiel>`,
  `matches:<spiel>[:all]`); `--daten <ordner>`
  kopiert einen Datensatz (etwa aus `testdaten.py`, gern mit einer
  `settings.json` für ein anderes Design) in beide Datenordner.
  Pause-Overlay und Highscore-Banner zeichnet der Prüfstand mit dem echten
  Code aus `main.py` (eine `main.App` ohne Tk).
  Nach dem Bild vergleicht es auch **alle gespeicherten Daten** beider
  Läufe – `mem.json`, `settings.json`, `ugc.json`, `replay.json`, Statistik,
  Match-Historie (der Prüfstand fährt die Match-Logik von `main._frame` und
  endet wie `beenden`) und was nach „Downloads“ geteilt wurde (PGN, Bahnen,
  Level, Replays);
  Zeitstempel der Wanduhr werden dabei nur in den Ziffern maskiert.
  `--nur-bild` schaltet das ab.
* **`regression.py [--aufloesung WxH] [--sprache xx] [spiel ...|screens]`** –
  feste Ereignisfolgen für alle 50 Spiele (mehrere Modi je Spiel) und alle
  Bildschirme; nach Änderungen an `py.rs`, `ui.rs`, `pg.rs` oder der App
  immer laufen lassen. Mit `--aufloesung` in jeder Stufe der
  Auflösungsliste (480×360 bis 1280×960), mit `--sprache` in jeder der
  14 Sprachen – Python und Rust lesen beides aus dem Datenordner.
  Läufe, die vom Datum abhängen (gestriges Tages-Sudoku lösen, gestriges
  Tageswort erraten), bauen ihre Ereignisse zur Laufzeit mit der
  Python-Referenz.
* **`bots/bot_<spiel>.py`** – Bots spielen in Python echte Partien
  (Tetris bis Level 12, Schach, Casino, Minigolf mit Editor, Geometry Dash
  mit Vorausschau-KI, Editor und Bibliothek, Lama Mines bis zum Höchstgewinn
  und mit Auto-Modus, …) und schreiben ihre Eingaben
  als Tastenfolge; `vergleich.py --keys-file` spielt sie in beiden Fassungen
  nach.
* **Abgleich einzelner Kerne** gegen Python: `chess_check.py` (Schach-Engine),
  `geodash_check.py` (Geometry-Dash-Physik über alle Level-Lösungen),
  `swear_check.py` (Wortfilter), `ui_probe.py` (ui.rs in 10 Designs).
* **`audit.py`** – jeder Übersetzungsschlüssel in allen 14 Sprachen,
  Platzhalter nur mit `tf`, Spieleliste gegen `defs.rs`, keine Umlaute im
  Quelltext.
* **`tasten_check.py`** – die Tastatur im echten Windows-Weg gegen Tk:
  dieselben Tastennachrichten gehen an ein (verstecktes) Tk-Fenster und an
  `pygamez.exe` mit `PYGAMEZ_TASTEN_PROBE` (verstecktes Fenster mit der
  Tastatur-Fensterprozedur der App); verglichen werden Tastenname und
  Zeichen von 91 Tasten im Layout des Rechners. Kein Fokus, nichts wird
  getippt.
* **`stats_check.py`** / **`import_check.py`** – was in der App nur über
  Datei-Dialoge geht: Statistik exportieren/importieren (7 Fälle) und
  geteilte Bahnen, Level und Replays importieren – über Kreuz (Python liest
  Rust-Dateien und umgekehrt, 11 Fälle), jeweils mit Vergleich der Daten.
* **`lamabank_check.py`** – die Lama-Bank (gemeinsames Chip-Konto der sechs
  Casino-Spiele) Buchung für Buchung gegen `lamabank.py`: Einsatz und
  Gewinn in einem Zug (`bet_and_pay`), Pleite und Kredit je Spiel, alter
  Spielstand, Migration, Poker-Tischstapel (5 Fälle, Antworten und
  `mem.json` gleich).
* **`matches_check.py`** – Match-Historie und Tages-Archiv Befehl für Befehl
  gegen `matches.py`/`dailyarchive.py` (Beispiel `matches_io` in
  `pgz-engine`): Kappe, Summen, Ersetzen, Bereinigen kaputter Einträge und
  Dateien, Sicherung `.bak`, Profile, Archiv-Zeilen, Datumstexte, Seeds,
  `date.fromisoformat` (371 Befehle, Antworten und Datei gleich).
* `vergleich.py` (und damit `regression.py`), `stats_check.py`,
  `import_check.py`, `lamabank_check.py` und `matches_check.py` nehmen die `.exe` aus
  `CARGO_TARGET_DIR`, falls gesetzt (eigener Bauordner), sonst aus
  `rust.output/`.
* **`vollbild_check.py`** – das echte Fenster: Startgröße wie `main.py`,
  Programm-Symbol, Vollbild über F11 ein und über den Knopf der
  Seitenleiste wieder aus, Mindestgröße 880×560 (Schalter
  `PYGAMEZ_VOLLBILD_PROBE`). Es öffnet für etwa zwei Sekunden ein Fenster.
* **`tempo.py [--aufloesung 1600x1000]`** – Millisekunden je Bild für alle
  Spiele (langsamstes: Blockjump mit rund 11 ms bei 1600×1000).
* `cargo run -p pgz-games --release --example music_check` – der
  Musikpfad (Geometry Dash), den der Prüfstand nie betritt; spielt stumm.
* `cargo run -p pgz-engine --release --example rumble_check` – die
  Gamepad-Vibration (XInput) mit Haptik an/aus.
* `cargo test --workspace --release` und `cargo clippy --workspace`
  (`clippy.toml` verbietet `serde_json::Map::remove`, siehe unten).

Der Prüfstand fasst nie die echten Spielstände an: Python und Rust bekommen
je einen frischen Datenordner, Replays und eigene Level werden dorthin
umgelenkt, Dateidialoge sind abgeschaltet und „Downloads" liegt ebenfalls im
Datenordner.

## Stand

* **50/50 Spiele** pixelgleich, dazu alle Bildschirme der App,
  Startbildschirm (alle 10 Designs), Pause-Overlay und Highscore-Banner.
* Match-Historie (`matches.pgzlamamatches`, Matches-Bildschirm mit Kurve,
  Filter, Sortierung) und Tages-Archiv (Crossy Road, Sudoku, Wordle:
  Datums-Leiste, Kalender, Archiv-Tage ohne Einfluss auf Serie und
  Statistik) wie in Python – Bild und Daten.
* Gesamt-Regression (233 Läufe: alle Spiele in mehreren Modi und alle
  Bildschirme) ohne Abweichung in Bild **und** gespeicherten Daten – in
  allen 14 Sprachen und in jeder Stufe der Auflösungsliste von 480×360 bis
  1280×960 (die Läufe für Match-Historie und Tages-Archiv in 480×360,
  640×480 und 1280×960 sowie de/en/fi/pl/fr). Dazu gehören Teilen/Export (Schach-PGN, Minigolf-Bahn,
  Geometry-Dash-Level, Replay) und Import ohne Dialog.
* Lama Plinko zusätzlich mit eigenen Spielständen (`vergleich.py --daten`):
  Jackpot ×1000 und MEGA-Gewinn (Fairness-Zustand mit gesuchter Nonce
  vorgegeben), Pleite beim Start, beschädigter Spielstand, Fortsetzen.
* Was nur über Datei-Dialoge geht, ist direkt gegen Python geprüft:
  Statistik-Export/-Import, Import geteilter Bahnen, Level und Replays.
* Das Fenster wie `main.py`: Startgröße nach der Bildschirmhöhe,
  Mindestgröße 880×560, Logo als Programm-, Fenster- und Taskleisten-
  Symbol, DPI-bewusst (scharf bei 125/150 %), Vollbild per F11 bzw. Knopf
  (randlos auf dem aktuellen Monitor), Alt+F4 schließt.
* Tastatur wie Tk – Tastennamen und Zeichen im Layout des Rechners (auf
  deutscher Tastatur: Y/Z, Umlaute, ß, Ziffernblock), Wiederholung wie bei
  Windows.
* Gamepad-Vibration (Option „Haptik“) über XInput – wie pygame/SDL für
  Xbox-kompatible Controller.
* `cargo clippy` ohne Fehler; die verbliebenen Hinweise sind Stilfragen
  (z. B. Index-Schleifen, die der Python-Vorlage folgen).

Bekannt und harmlos: Der Minigolf-Editor rechnet in Python mit wechselnden
Zahltypen (`20` bzw. `20.0`), Rust durchgehend mit Kommazahlen – Python lädt
und spielt Rust-Bahnen unverändert (geprüft), nur die Zahlen in der Datei
sehen anders aus. Nicht automatisch prüfbar sind echte Vibration (ohne
Controller) und Dateidialoge selbst.

## Regeln beim Übertragen

* **Python ist die Vorlage.** Konstanten, Reihenfolge der Zeichenaufrufe und
  Zufallsaufrufe wie in Python; Zahlen-Literale genau so (auch `6.28`).
* Pythons `//`, `%`, `round()` und `int()` haben andere Regeln als Rust –
  immer die Helfer aus `py` nehmen (`py::fdiv`, `py::pmod`, `py::round`,
  `py::ffdiv`, `py::fpmod`); Achtung bei `-r // 3` (= `(-r) // 3`).
* Reihenfolgen, die Python über ein `dict` oder `set` erzeugt, mit
  geordneten Strukturen nachbilden (`serde_json` mit `preserve_order`,
  `indexmap`, `py::IntSet`, `py::CellSet`). **Nie `Map::remove`** – das ist
  dort ein `swap_remove` und zieht den letzten Eintrag nach vorn; Pythons
  `dict.pop`/`del` ist `shift_remove` (`clippy.toml` verbietet `remove`).
* Dateien wie Python im Textmodus schreiben: `store::text_mode` (unter
  Windows `\r\n`), Stempel `_generated` wie Python (ein vorhandener bleibt).
* `Font::render` liefert ein geteiltes Bild – vor `set_alpha` immer `.copy()`.
* Kommentare deutsch ohne Umlaute, sichtbare Texte nur aus `lang/`.
