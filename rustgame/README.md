# rustgame

**pygame-ce 2.5.7, in Rust nachgebaut.** Gleiche API, gleiches Verhalten —
und zwar nicht ungefähr, sondern **Pixel für Pixel nachgewiesen**.

Das ist etwas anderes als der Ordner `rust/` daneben. Der portiert die
*Spiele* von PyGameZ auf macroquad und hat bewusst keine Surface-Abstraktion.
`rustgame` baut die **Engine** nach, auf der PyGameZ läuft — damit man darauf
aufbauen kann, statt daneben.

---

## Benutzen

Man braucht **Rust** (stable, über rustup) und ein **Python ab 3.11** —
die Hülle ist `abi3`, ein Bau läuft auf jedem Python ab 3.11. Ein
installiertes pygame-ce braucht es nicht, numpy nur für `surfarray` und
`sndarray` (wie bei pygame). Gemessen ist alles unter Windows 11; Linux
und macOS sind es nicht.

```bash
python rustgame/tools/einbauen.py            # bauen und in die site-packages dieses Pythons legen
python rustgame/tools/starten.py spiel.py    # ein unverändertes pygame-Programm auf rustgame
```

`einbauen.py` legt `rustgame_py.pyd` und seine zwei Begleitdateien
(`freesansbold.ttf`, `pygame_icon.bmp`) in die `site-packages` des Pythons,
mit dem man es ruft. `starten.py` lässt `import pygame`, `from pygame.locals
import *` und `import pygame.freetype` auf rustgame zeigen — mit derselben
Umleitung, unter der pygames Testsuite und die 46 Spiele von PyGameZ
gemessen sind. Wer sein Programm selbst anpasst, schreibt `import rustgame_py
as pygame`; das genügt, solange es nur über diesen einen Namen geht.

Fenster und Ton sind echt (minifb, rodio). `SDL_VIDEODRIVER=dummy` und
`SDL_AUDIODRIVER=dummy` schalten wie bei pygame auf fensterlos und stumm.
Beim Laden grüßt rustgame mit pygames Zeile (`pygame-ce 2.5.7 (SDL
2.32.10, Python …)`), und `PYGAME_HIDE_SUPPORT_PROMPT` schaltet sie ab.

**Wo es nicht pygame ist** — alles gemessen und unten benannt:

| Stelle | wie weit |
|---|---|
| Alpha-Blit auf Flächen mit 24, 16, 15 oder 8 Bit | eine Stufe daneben, bei 8 Bit mit Farbtafel weit. Auf 32 Bit (Bildschirm, `convert`, `convert_alpha`, `SRCALPHA`) gleich |
| JPEG lesen | andere Pixel: 39 von 41 Dateien, 2 bis 61 Stufen |
| JPEG und PNG schreiben | JPEG mit anderem Kodierer, PNG mit denselben Pixeln, aber anderer Kompression |
| Windows-Bitmapschriften `.fon` | lädt rustgame nicht |
| Text | gleich, aber deutlich langsamer (Schach-Audit 293 s gegen 34 s) |

---

## Warum „nachgewiesen" hier ein starkes Wort ist

Ein Nachbau, der „ungefähr gleich aussieht", ist wertlos: die Abweichungen
zeigen sich dann irgendwann mitten in einem Spiel, und niemand weiß, welche
der zehntausend Zeilen schuld ist. Deshalb steht hier vor jeder Zeile Rust
eine **Messung an echtem pygame**.

### Stufe 1 — Die Mischformel, vollständig durchgerechnet

`tools/verify_alpha.py` hat **alle 16.777.216** Kombinationen von (Ziel,
Quelle, Alpha) durch echtes pygame geschickt und gegen mehrere Kandidaten
geprüft. Ergebnis:

| Formel | falsche Ergebnisse |
|---|---:|
| die Lehrbuchformel `(s·a + d·(255−a)) / 255` | 8.126.464 |
| SDLs eigenes Makro | 3.342.336 |
| **`(s·(a+1) + d·(256−a)) >> 8`** | **0** |

Die richtige Formel steht in `blend_formula.rs` — erzeugt vom Skript, nicht
abgetippt.

Und es ist nicht bei einer Formel geblieben. Für denselben Aufruf
`surface.blit(..., special_flags=0)` hat pygame **vier** Rechenwege, und sie
liefern verschiedene Zahlen:

| Quelle | Schlüsselfarbe | Routine | `d=100 s=50 a=183` |
|---|---|---|---:|
| mit Alphakanal | nein | pygames `alphablit_alpha` | 64 |
| mit Alphakanal | ja | SDLs `SDL_Blit_Slow` | 63 |
| ohne Alphakanal | ja | SDLs `BlitNtoNSurfaceAlphaKey` | **65** |
| ohne Alphakanal | nein | SDLs `BlitRGBtoRGBSurfaceAlpha` | 64 |

Drei verschiedene Ergebnisse für dieselben drei Zahlen. Der Unterschied ist
jeweils ein einziger Rundungsschritt — `>> 8` gegen `/ 255`, eine Summe
gegen zwei einzeln geteilte Summanden, und bei negativen Zwischenwerten
Abrunden gegen Abschneiden zur Null hin.

Das war **nicht** aus Messreihen zu rekonstruieren: die Daten widersprachen
sich, weil jede Messreihe zufällig eine andere Routine traf. Erst der Blick
in `alphablit.c`, `SDL_blit_A.c` und `SDL_blit_slow.c` hat es aufgelöst.

### Stufe 2 — 123 Referenzbilder

`tools/cases.py` beschreibt jede Szene **einmal**; `gen_golden.py` rendert
sie mit echtem pygame nach `tests/golden/*.png` **und** übersetzt dieselbe
Szene nach Rust. Es gibt also keine zwei Listen, die auseinanderlaufen
können.

```
cargo test --test golden     ->  123 Fälle pixelweise geprüft
```

Weicht ein Pixel ab, landet ein **Dreitafelbild** (erwartet ‖ bekommen ‖
Unterschied achtfach verstärkt) in `rustgame.output/golden-diff/`, und die
ersten zehn abweichenden Stellen werden mit beiden Farbwerten genannt.

### Stufe 3 — Differenzielles Fuzzing

Referenzbilder prüfen je *eine* Sache. Fehler entstehen aber im
**Zusammenspiel** — ein Blit mit Schlüsselfarbe auf eine beschnittene
Teilfläche, deren Elternteil ein Flächen-Alpha trägt. So etwas schreibt
niemand von Hand als Testfall auf.

`tools/fuzz_ops.py` würfelt es: echtes pygame führt die Folge aus, abgelegt
wird **nur der Hash** (1200 Fälle wären sonst 1200 PNGs).

```
.venv/Scripts/python.exe rustgame/tools/fuzz_ops.py       # 600 Fälle
cargo test --test fuzz
```

**Das hat sich sofort ausgezahlt.** Beim ersten Lauf fielen 18 von 120
Fällen — obwohl alle Referenzbilder grün waren. Gefunden wurden dabei unter
anderem:

- Ein `area`-Rechteck, das links oder oben aus der Quelle ragt, **verschiebt
  das Ziel mit**. Nur rechts und unten wird gekürzt.
- `scroll` arbeitet **nur innerhalb des Clip-Rechtecks** — und verschiebt
  roh, statt zu blitten.
- Die Schlüsselfarbe gilt **nur beim normalen Blit**; alle zwölf
  Mischarten übergehen sie.
- Eine Quelle ohne Alphakanal liefert den `BLEND_RGBA_*`-Mischarten das
  Alpha 255 — sobald aber `set_alpha` auch nur **einmal** aufgerufen wurde,
  das Alpha **0**. Unabhängig vom übergebenen Wert.

### Das unsichtbare vierte Byte

Eine Fläche ohne Alphakanal hat trotzdem vier Bytes je Pixel, und das
vierte ist **echter Zustand** — `get_at` und `tobytes` maskieren es auf
255, aber es steht etwas anderes drin:

| Schreibweg | viertes Byte |
|---|---|
| `fill`, `draw`, `set_at`, normaler Blit | `00` |
| Blit mit `set_alpha` unter 255, ohne Schlüsselfarbe | `ff` |
| `BLEND_RGBA_ADD`, `-MAX`, `PREMULTIPLIED` | der gerechnete Wert |
| `BLEND_RGBA_SUB`, `-MIN`, `-MULT` | `00` (aus der Null gerechnet) |

Gefunden nur, weil `scale2x` und `flood_fill` **ganze 32-Bit-Wörter**
vergleichen statt sichtbarer Farben. Jede einzelne Zeile ist am
Rohspeicher nachgemessen.

Fällt ein Fall, zerlegt `--zerlege <nummer>` ihn in Präfixe. Dann sagt der
Test nicht mehr, *welcher Fall* kippt, sondern **welche Operation**.

**Eine Lücke steht noch offen, und sie ist benannt.** Etwa 2 von 5000
Fällen weichen ab, alle mit demselben Muster: `scale2x` nach einer
`BLEND_RGBA_*`-Mischart. `scale2x` vergleicht **ganze 32-Bit-Wörter**,
also einschließlich des vierten Bytes — und auf einer Fläche ohne
Alphakanal ist dieses Byte unsichtbarer Zustand: `get_at` und `tobytes`
maskieren es weg, aber eine RGBA-Mischart schreibt hinein.

Was pygame dort genau ablegt, ließ sich **nicht eindeutig messen**: zwei
Versuchsreihen widersprechen einander — in der einen liest pygame das
Quell-Alpha einer Fläche ohne Alphakanal als 255, in der anderen roh als 0.
Solange das nicht geklärt ist, wäre jede Umsetzung geraten, und Raten ist
hier genau die Sache, die nicht passieren darf.

Deshalb zählt der Test diese Fälle **getrennt und benennt sie**, statt
dauerhaft rot zu sein — ein roter Test würde jede *neue* Abweichung
verdecken, und das wäre schlimmer als die Lücke.

### Stufe 4 — Dasselbe Skript, beide Engines

Am Ende zählt nur eine Frage: läuft ein pygame-Programm, das niemand
angefasst hat, durch die Rust-Engine hindurch? `tools/vergleich_py.py`
beschreibt eine Szene **einmal** und übergibt sie beiden Seiten mit ihrem
eigenen Modul:

```
.venv/Scripts/python.exe rustgame/tools/vergleich_py.py
->  0 von 9600 Pixelbytes unterschiedlich
```

Die Szene enthält gerundete Rechtecke, Umrisse mit Strichstärke, Kreise
gefüllt und offen, Linien, Polygone, Ellipsen, einen halbdurchsichtigen
Blit, Spiegeln, Skalieren und `get_rect(center=…)`.

### Was hier nicht geht — und warum

| Sache | Grenze | Grund |
|---|---|---|
| `smoothscale` | **keine mehr** (Runde 18) | Stand hier als „±2 je Kanal, pygame ist nicht einmal mit sich selbst pixelgleich". Beide Rechenwege (`SSE2`, die Vorgabe, und `GENERIC`) sind jetzt übertragen und treffen pygame je Byte für Byte. |
| Text über `pygame.font` | **keine mehr** (Runde 18) | Stand hier als „Rasterung ±1 px". Gemessen waren **0 von 11.984** Zeichenbildern gleich. Jetzt gehintet (`skrifa`), gerastert mit übertragenem `ftgrays`, gesetzt mit HarfBuzz (`harfrust`): 11.984 von 11.984 Zeichen, 6.000 von 6.000 Sätzen aus PyGameZ, alle 46 Spiele mit echtem Text Bild für Bild gleich. |
| AIFF, VOC, LBM `PBM `, XV | **gibt es** (Runde 19) | Stand hier als „fehlen“. Übertragen aus SDL_mixer `load_aiff.c` und `LoadAIFFMusic` (AIFF und AIFC samt `sowt`, A-law, µ-law, `fl32`/`fl64`) und `load_voc.c`, aus SDL_image `IMG_lbm.c` (ILBM und PBM, EHB, HAM), `IMG_gif.c` (der eigene LZW-Leser), ICO/CUR und XV. Kaputte Dateien sagen in allen Formaten SDLs Sätze. |
| Sätze der Musik-Dekodierer | **eigene**, vier Fälle | `music.load` einer kaputten OGG-, FLAC-, MP3- oder WavPack-Datei nennt bei pygame den Dekodierer (`stb_vorbis_open_rwops: VORBIS_invalid_setup`, `music_drflac: …`). Hier stehen andere Dekodierer. Die `XMP:`-Sätze (unbekannte Daten, kaputtes Trackermodul) sind seit Runde 19 nachgebaut; auf dem Weg von `Sound` war der Satz schon gleich. |
| `pygame.freetype` mit zwei Achsen | **keine mehr** (Runde 20) | Stand hier als „anders“: FreeType hintet `size=(24, 12)` mit gestrecktem TrueType-Interpreter, skrifa kannte eine Skala. Der gestreckte Modus steckt jetzt im skrifa-Abzug; gemessen 8760 von 8760 Fällen an zwölf Windows-Schriften und 8176 von 8176 an pygames Testschriften. |
| Windows-Bitmapschriften `.fon` | **fehlen** (Runde 20) | pygame lädt 189 `.fon` des Schriftordners über FreeTypes `winfnt`, rustgame lehnt sie ab. `sysfont` nimmt sie nicht auf. |
| Alpha-Blit auf weniger als 32 Bit | **anders** (Runde 21) | Bei pygame mischt dort SDLs eigener Blitter (`SDL_BlitA.c`), nicht `alphablit.c`. Gemessen bis jetzt nur auf 32 Bit — bis die Frage „ist alles da“ eine 24-Bit-Fläche traf. |
| JPEG lesen und schreiben | **anders** (Runde 21) | SDL_image rechnet mit libjpeg-turbo, rustgame mit `zune-jpeg` und `jpeg-encoder`. Größe, Tiefe und Flaggen stimmen, die Pixel nicht. |
| PNG schreiben | **andere Bytes** (Runde 21) | Dieselben Pixel und Blöcke, andere Kompression (pygame: libpng und zlib). |
| `Rect` und `Color` | sind `Copy` | pygame hat Referenzsemantik (`a = b` teilt dasselbe Objekt). In der Python-Hülle stimmt es wieder, weil dort echte Objekte entstehen. |
| Puffer-Protokoll | **gibt es**, in jeder Tiefe | Stand hier zweimal als Grenze: erst „nur die Adresse, nicht die Struktur", dann „außer bei 8, 16 und 24 Bit". Beides ist weg. `memoryview(s.get_buffer())`, `__array_struct__`, `pixelcopy`, `surfarray` und `sndarray` gehen; der Pixelspeicher hält Bytes im gemeldeten Format, der Klangspeicher im Ausgabeformat des Mixers. Das `unsafe` dafür steht in `puffer.rs`; fünf Dateien reichen die zwei PyO3-Schlitze durch, sonst gilt überall `forbid`. |
| Paletten | **gibt es** (8 Bit mit Farbtafel) | Stand früher hier als „entfallen, PyGameZ benutzt ausschließlich 32 Bit" — richtig für PyGameZ, falsch für pygames Testsuite, die Palettenflächen an Dutzenden Stellen prüft. Ein Vermerk über den *Bedarf* ist keiner über die *Fähigkeit*. |
| Geräte (Kamera, MIDI) | **gibt es** | Stand dreimal als Grenze: „braucht den Gerätezugriff des Betriebssystems, und der ginge nur über FFI". Der erste Halbsatz war wahr, der zweite dieselbe Verwechslung wie bei `get_total_ram`: `unsafe` braucht **irgendwer**. `nokhwa` und `midir` tun es hinter einer sicheren Schnittstelle. |
| Flächen über Fadengrenzen | **gehen** | Stand hier als `unsendable` — „eine `Surface` teilt ihren Speicher über `Rc<RefCell<..>>`". Der Plan sah den Wechsel auf `Arc<..>` vor und nannte ihn „eine Zeile"; fällig wurde er durch einen Fehler, der würfelte. Es waren drei Aliase und knapp siebzig Stellen. |

---

## Aufbau

```
rustgame/
  crates/
    rustgame-core/      Surface, Rect, Color, Blit, draw, transform, image, time
    rustgame-font/      Text: skrifa-Hinting (TrueType und CFF), übertragenes
                        ftgrays/ftraster/FT_Stroker, HarfBuzz, pygame.freetypes
                        Zeilenaufbau, sysfont
  vendor/skrifa/        skrifa 0.48.0 mit drei rustgame-Zusätzen (siehe dort)
    rustgame-display/   Fenster und Ereignisse über minifb + Dummy-Backend
    rustgame-mixer/     Ton über rodio
    rustgame-math/      Vector2/Vector3 (`pygame.math`) und `pygame.mask`
    rustgame-sprite/    Sprites und Gruppen — die Rust-eigene Fassung
    rustgame-py/        die Python-Hülle (PyO3) — bewusst außerhalb des Workspace
      src/puffer.rs       das Puffer-Protokoll — hier liegt das `unsafe`
      src/python/         wörtlich übertragene pygame-Module (Python)
  tests/golden/         123 Referenzbilder aus echtem pygame (eingecheckt!)
  tools/                die Messwerkzeuge (siehe unten)
  API.md                Abdeckungstabelle — erzeugt, nicht gepflegt
```

`rustgame-py` steht **absichtlich** nicht in der Mitgliederliste des
Workspace: PyO3s `extension-module` bindet kein libpython ein, damit ließe
sich kein Testprogramm mehr linken. Als Mitglied hätte die Kiste
`cargo test --workspace` für alles andere zerschossen.

Getrennte Kisten statt Features, damit der Kern **ohne jede
Betriebssystem-Abhängigkeit** baut und kopflos testet — genauso wie die
pygame-Referenz, gegen die geprüft wird.

Die Referenzbilder sind **eingecheckt**. Sie sind der Vertrag; wer sie neu
erzeugt, ändert die Messlatte und muss das begründen.

**Eine Ausnahme von „die Hülle reicht nur durch":** `pygame.sprite`.
`figur.rs` ist keine Hülle um `rustgame-sprite`, sondern eine
Übertragung von `pygame/sprite.py`. Der Grund ist eine einzige gemessene
Zeile — `self.rect.x += 1`. Python macht daraus
`tmp = self.rect; tmp.x = tmp.x + 1` und schreibt nichts zurück; das
wirkt **nur**, wenn `sprite.rect` jedes Mal dasselbe Python-Objekt ist.
Ein Zugriff, der aus dem Kern ein frisches `PyRect` baut, macht die
häufigste Zeile jedes pygame-Spiels wirkungslos. Die Rust-Kiste bleibt
die Rust-eigene Sprite-Welt und wird vom Workspace mitgeprüft.

### Die Werkzeuge

| Datei | wofür |
|---|---|
| `verify_alpha.py` | nagelt die Mischformel exhaustiv fest — **läuft vor allem anderen** |
| `cases.py` | die Falltabelle: **eine** Quelle für Python und Rust |
| `gen_golden.py` | erzeugt Referenzbilder **und** den Rust-Test daraus |
| `fuzz_ops.py` | würfelt Operationsfolgen, vergleicht Hashes |
| `api_coverage.py` | fragt das echte pygame **und die gebaute Hülle** per `dir()` ab und schreibt `API.md` |
| `vergleich_py.py` | die Abnahme: dasselbe Skript auf beiden Engines |
| `einbauen.py` | baut die Hülle und legt sie **an jede Stelle**, von der sie geladen wird — mit SHA-256-Nachweis |
| `uebersprungen.py` | fährt die Testsuite **zweimal** und vergleicht, *welche* Tests jede Engine überspringt — nicht wie viele |
| `abdeckung.py` | schreibt mit `sys.monitoring` jeden Aufruf mit und sagt, **welche Funktion hier noch niemand angefasst hat** |
| `deutsche_fehlertexte.py` | ein Wächter: **0**, und er endet mit 1, sobald ein deutscher Satz in eine Ausnahme kommt |
| `crates/rustgame-py/tools/vergleich_module.py` | **862** Einzelproben an beide Engines, Wert für Wert; `--genau <Teil>` zeigt eine Abweichung **ganz** |
| `starten.py` | lässt ein **unverändertes** pygame-Programm auf rustgame laufen (`import pygame` zeigt auf die Hülle) |
| `alle_schriften.py` | jede Schrift des Windows-Schriftordners, beide Schriftmodule, beide Engines — jeder Block im eigenen Prozess, weil pygame dabei abstürzt |
| `crates/rustgame-py/tools/vergleich_spiel.py` | ein ganzes Spielprogramm, Byte für Byte — 30 Bilder, auf Wunsch **300** (die Zahl aus dem Plan) oder 1000 |
| `crates/rustgame-py/tools/gen_konstanten.py` | erzeugt die 589 Konstanten aus der einen Quelle |
| `crates/rustgame-py/tools/gen_zeiger.py` | schneidet `pygame/cursors.py` aus — 430 Zeilen Zeigerdaten, die niemand abtippen sollte |

**numpy gehört dazu.** `pygame.surfarray` und `pygame.sndarray` sind in
pygame selbst reines Python über numpy; ohne numpy steht dort ein
`MissingModule`, das bei jedem Zugriff einen `NotImplementedError` wirft.
rustgame macht es genauso. Zum **Prüfen** ist numpy trotzdem nötig, und
zwar nicht nur für die zwei Module: `pixelcopy_test.py`, `base_test.py`
und `surface_test.py` löschen ohne numpy ganze Testklassen aus sich
selbst, und die fehlten dann in der Zahl. `pip install numpy` in derselben
venv genügt.

`gen_golden.py` erkennt **selbst**, welche Rust-Module schon fertig sind: es
sucht den Satz `**Noch nicht umgesetzt.**` im Dateikopf. Solange er dort
steht, werden die zugehörigen Fälle übersprungen statt rot — die Abdeckung
wächst sichtbar, ohne die Testsuite zu blockieren, und niemand muss eine
Liste nachpflegen.

---

## Bauen und prüfen

```bash
cd rustgame
cargo test   --workspace --release               # 651 Tests, Referenzbilder, Fuzzer
cargo clippy --workspace --all-targets           # ohne Warnung
cargo doc    --workspace --no-deps               # ohne Warnung
cargo run -p rustgame-display --example demo     # echtes Fenster
```

**Und dieselben drei in der Hülle noch einmal** — sie hat eine eigene
`[workspace]`-Tabelle und ist in keinem `--workspace` enthalten. Genau
deshalb standen dort lange 3 Typfehler, 18 Clippy- und 15 Doku-Hinweise:

```bash
cd rustgame/crates/rustgame-py
cargo test   --release --all-targets             # 12 Tests
cargo clippy --release --all-targets             # ohne Warnung
cargo doc    --no-deps                           # ohne Warnung
```

Die Build-Ausgabe landet in `rustgame.output/` **neben** dem Quellordner —
so bleibt `rustgame/` frei von Build-Müll (genauso wie `rust/` es mit
`rust.output/` hält).

Die Messwerkzeuge brauchen das echte pygame aus der venv des Projekts:

```bash
.venv/Scripts/python.exe rustgame/tools/einbauen.py          # bauen + legen
.venv/Scripts/python.exe rustgame/tools/gen_golden.py
.venv/Scripts/python.exe rustgame/tools/fuzz_ops.py --anzahl 1200
.venv/Scripts/python.exe rustgame/tools/api_coverage.py
.venv/Scripts/python.exe rustgame/tools/vergleich_py.py
.venv/Scripts/python.exe rustgame/crates/rustgame-py/tools/vergleich_module.py
.venv/Scripts/python.exe rustgame/crates/rustgame-py/tools/vergleich_spiel.py 300
.venv/Scripts/python.exe rustgame/crates/rustgame-py/tools/pygame_tests.py
.venv/Scripts/python.exe rustgame/tools/uebersprungen.py    # welche, nicht wie viele
```

**`einbauen.py` zuerst, und nichts anderes.** Wer `cargo build` von Hand
ruft und die `.pyd` selbst kopiert, hat eine von drei Fallen vor sich —
siehe „Die Falle mit der gebauten Datei".

---

## Wie weit es ist

**API-Abdeckung: 3471 von 3471 Namen — 100 %.**

**Pygames eigene Testsuite: 2364 von 2485 erreichbaren Tests, aus
54 Testdateien — null Abweichungen.**

Beide Nenner sind dabei **gewachsen**, und das ist der eigentliche
Fortschritt: die Messgeräte fragen mehr als vorher.

| | vorher | jetzt | was nie gefragt wurde |
|---|---:|---:|---|
| API-Namen | 768 | **3471** | erst 317 Klassen und Module der Untermodule, dann die **70 Namen des Hauptmoduls** selbst, dann **fünf Untermodule von `_sdl2`**, dann **neun Untermodule, die sich nicht einmal importieren ließen**, dann **jede Konstante und jeden mitimportierten Namen jedes Untermoduls** — und zuletzt **21 Klassen, die in keiner Liste standen** |
| Testdateien | 51 | **54** | `debug_test.py`, `docs_test.py`, `image__save_gl_surface_test.py` |
| Proben im Modulvergleich | 611 | **820** | `newbuffer`, `pypm`, `pkgdata` und — als je eine Probe — **`dir()` gegen `dir()` für alle 60 Untermodule**, **die Basisklassen aller 65 Klassen**, **`dir()` des Hauptmoduls in beide Richtungen** und **`from X import *` für alle 61** |
| Tests in der Hülle (`rustgame-py`) | 0 | **12** | die Kiste steht außerhalb des Workspace; `cargo test` lief dort **nie** |

Eine Zahl, die steigt, weil weniger gefragt wird, ist schlimmer als eine,
die fällt, weil mehr gefragt wird. Siehe unten, „Der wiederkehrende
Fehler".

Die Testzahl ist auf diesem Weg zweimal **gefallen** — von 98 % auf 91 %
und später von 97 % auf 94 % —, ohne dass je ein Test schlechter geworden
wäre. Jedes Mal hatte das Messgerät vorher zu wenig gefragt:

```text
api_coverage.py   fragte 30 Module       pygame hat 36
api_coverage.py   fragte kein dir(pygame)
api_coverage.py   fragte keine Konstanten der Untermodule
api_coverage.py   fragte 24 Klassen        erreichbar sind 45
alle_klassen()    prueft __module__        Cython schreibt dort "builtins"
api_coverage.py   nahm sieben Namen aus    (os, sys, platform, warnings,
                                            ABC, abstractmethod, errorfnc)
pygame_tests.py   fuhr 24 Testdateien    der Ordner hat 50
pygame_tests.py   fuhr 47 Testdateien    (abgetippt, obwohl "gelesen")
vergleich_module  nahm acht Namen aus      (dieselbe Sorte Handliste)
vergleich_module  schnitt jede Abweichung bei 200 Zeichen ab
numpy fehlte      surfarray/sndarray ließen sich nicht einmal laden
```

`import pygame` lädt `gfxdraw`, `freetype`, `midi`, `camera` und
`_sdl2.video` **nicht** mit; wer sie nur über `getattr` sucht, findet
nichts. `sprite_test.py` hatte damals **141 grüne Tests**, die nie gezählt
wurden — und `geometry_test.py` 145 rote, von denen niemand wusste.

**Jetzt sind beide Seiten gelesen und nicht gepflegt:** die linke Spalte
der API-Tabelle kommt aus dem installierten pygame per `dir()`
(einschließlich `dir(pygame)` selbst), und `alle_dateien()` liest den
Testordner. Was dazukommt, hängt hinten dran und fällt auf.

Eine Zahl, die steigt, weil weniger gefragt wird, ist schlimmer als eine,
die fällt, weil mehr gefragt wird.

### Vollständig grün — alle 50 Dateien

`rect` (365), `mask` (261), `draw` (251), `math` (181), `geometry` (145),
`sprite` (143), `surface` (137), `color` (93), `ftfont` (63), `font` (63),
`transform` (60), `freetype` (52), `mixer` (50), `image` (50),
`event` (47), `pixelarray` (44), `display` (39), `render` (32),
`mixer_music` (30), `gfxdraw` (26), `window` (22), `surfarray` (19),
`rwobject` (19), `base` (18), `bufferproxy` (16), `time` (14), `blit` (13),
`pixelcopy` (12), `constants` (11), `mouse` (10), `key` (9),
`imageext` (9), `midi` (8), `sndarray` (7), `controller` (7),
`sysfont` (6), `touch` (5), `system` (5), `joystick` (5), `surflock` (4),
`video` (3), `version` (3), `cursors` (3), `typing` (1), `locals` (1),
`docs` (1), `debug` (1).

Drei Dateien überspringen sich **ganz**: `scrap_test.py` (die Ablage
braucht ein Fenstersystem), `camera_test.py` (seine eine Klasse ist leer —
`class CameraModuleTest(unittest.TestCase): pass`) und
`image__save_gl_surface_test.py` (braucht einen GL-Kontext). Bei pygame
überspringen sie sich ebenso.

**Fünf Dateien sind in diesem Zug grün geworden** — vier davon durch
Dinge, die unter dem Wort „Rasterkante" gelegen hatten:

| | vorher | jetzt | woran es lag |
|---|---:|---:|---|
| `freetype` | 49 | **52** | eingebettete Bitmapschnitte, die Schriftgröße, der Leerzeichenkasten, das Alpha, die zwei Stufen |
| `ftfont` | 62 | **63** | die Schriftgröße — doppelt angewandter Faktor 0,6875 |
| `debug` | — | **1** | `pygame.print_debug_info` gab es nicht, und die Datei wurde nie gefahren |
| `docs` | — | **1** | `pygame.docs` gab es nicht, dito |
| `image__save_gl_surface` | — | **(1)** | überspringt sich selbst, wurde aber nie gefragt |

**Und zwölf Dateien im Zug davor**, als Landkarte der beiden
Speicherumbauten:

| | vorher | jetzt | woran es lag |
|---|---:|---:|---|
| `surface` | 132 | **137** | Puffer auf 8/16/24 Bit |
| `pixelarray` | 39 | **44** | dito, plus der Tafelindex |
| `pixelcopy` | 0 | **12** | gab es vorher nur als Fehler |
| `surfarray` | — | **19** | gab es vorher nicht |
| `sndarray` | — | **7** | dito, plus das Klangformat |
| `mixer` | 47 | **50** | das Klangformat |
| `base` | 11 | **18** | das Modul gab es nicht |
| `bufferproxy` | 11 | **16** | Puffer-Protokoll |
| `color`, `mask` | 91/260 | **93/261** | Puffer-Ausgeber |
| `system` | 4 | **5** | `get_total_ram` über `sysinfo` |
| `transform` | 59 | **60** | eine Folgestelle des Umbaus |

### Die Module, die zuletzt dazugekommen sind

| Modul | Tests | |
|---|---:|---|
| `pygame._debug` | **1 / 1** | wörtlich übertragen, 186 Zeilen; `print_debug_info` |
| `pygame.docs` | **1 / 1** | sagt ehrlich, dass keine Doku mitgeliefert wird |
| `pygame.surflock` | **4 / 4** | ein Modul **ohne einen einzigen Namen** — gemessen |
| `pygame.camera` | **(leer)** | echter Gerätezugriff über `nokhwa`; die Testdatei ist leer |
| `pygame.midi` | **8 / 8** | echte Geräte über `midir` |
| `pygame.pixelcopy` | **12 / 12** | vier Namen, die vorher nur einen Fehler warfen |
| `pygame.surfarray` | **19 / 20** | wörtlich übertragen, 471 Zeilen |
| `pygame.sndarray` | **7 / 7** | wörtlich übertragen, 150 Zeilen |
| `pygame.base` | **18 / 18** | das Modul gab es vorher nicht |
| `pygame.ftfont` | **62 / 63** | `pygame.font` auf dem zweiten Schriftsystem |
| `pygame.font` | **63 / 63** | mit Kontur, Laufrichtung und Schriftsystem |
| `pygame.image` | **50 / 50** | mit SVG, XCF und Paletten-TIFF |
| `pygame.mixer.music` | **30 / 30** | mit OPUS, WavPack und XM |
| `pygame._render` | **32 / 32** | `Renderer`, `Texture`, `Image` |
| `pygame.freetype` | 44 / 52 | mit **BDF-Bitmapschriften** und Drehung |
| `pygame.pixelarray` | 36 / 44 | Start/Schritt statt Indexlisten |

`pygame.ftfont` ist dabei die günstigste Zeile des ganzen Projekts: 232
Zeilen Python, und `ftfont_test.py` fährt damit **die ganze
`font_test.py` ein zweites Mal** — gegen das zweite Schriftsystem. 61
Tests für eine Datei.

### Fünf Befunde aus dem Speicher-Zug

* **`get_rect` gibt den Tintenkasten, nicht den Zeilenkasten.**
  `get_rect("M")` ist bei Größe 24 `(1, 17, 18, 17)` — und das `y` ist
  kein Bildschirmversatz, sondern die Höhe der Tinte **über der
  Grundlinie**. `Font.pad = True` schaltet auf den Zeilenkasten zurück;
  das ist es, was `pygame.ftfont` einstellt, und gedreht geht es gar
  nicht (`padding is unsupported for rotated text`).
* **`get_sized_glyph_height` ist Auf- plus Unterlänge plus eins.**
  Nachgerechnet für sieben Größen. Der naheliegende Weg — die Umrandung
  aus der `head`-Tabelle mit dem Größenfaktor — trifft zwei von sieben.
* **SDL_ttf und FreeType runden die Unterlänge verschieden**, und beide
  haben recht: `TTF_FontDescent` rundet auf, `FT_Request_Metrics` ab.
  Dieselbe Schrift in Größe 24 gibt `-5` und `-6`. Wer eine Zahl für
  beide nimmt, hat an einer Stelle unrecht.
* **`pixelcopy.make_surface` gibt für jedes zweidimensionale Feld acht
  Bit**, auch für `=Q`. Wer aus einem `Uint32`-Feld eine 32-Bit-Fläche
  erwartet, irrt.
* **`surface_to_array(.., "A")` gibt 255, sobald die Fläche nicht
  mischt.** Nach `set_alpha(None)` ist auch eine `SRCALPHA`-Fläche
  undurchsichtig — `surfarray_test` sagt es im Fehlertext.

### Sechs Befunde aus dem Puffer-Zug

Die vollständige Herleitung steht unten unter „Das Puffer-Protokoll";
hier die sechs, die man sich nicht ausdenkt.

* **`get_buffer()` sperrt die Fläche sofort, `get_view(...)` erst beim
  ersten Zugriff.** Gemessen: nach `s.get_view("2")` ist `get_locked()`
  falsch, nach `v.length` wahr, und nach `v = None; gc.collect()` wieder
  falsch. Die Sperre ist der `BufferProxy` **selbst** — `s.get_locks()`
  gibt ihn heraus.
* **`BufferProxy(objekt)` fragt beim Anlegen nichts.** `BufferProxy(1)`
  geht durch; erst `bp.length` gibt den `ValueError`. Und
  `base_test.test_PgObject_GetBuffer_array_interface` prüft es von der
  anderen Seite: direkt nach `BufferProxy(o)` darf `o` noch nicht gefragt
  worden sein.
* **`PyBUF_ND` allein scheitert, obwohl `PyBUF_SIMPLE` durchgeht.**
  `SIMPLE` fragt nach einem Stück Speicher, und das ist die Fläche; `ND`
  fragt nach einem Feld ohne Schrittweiten, und dafür müssten die Zeilen
  lückenlos aneinanderliegen. Die zweidimensionale Sicht ist
  **spaltenweise** zusammenhängend, weil `shape` `(Breite, Höhe)` ist.
* **Ein Format geht wörtlich durch.** `"1h"` heißt dasselbe wie `"h"`,
  kommt aber unverändert zurück; `">h"` trägt eine gedrehte Byteordnung;
  `"hh"` ist ungültig und muss ein `ValueError` werden. Eine Tabelle
  statischer Formate kann keines der drei — hier stand eine.
* **`pixelcopy.make_surface` gibt für jedes zweidimensionale Feld eine
  Fläche mit acht Bit**, ganz gleich, wie breit seine Einträge sind.
  Gemessen für `B`, `=H`, `=I` und `=Q` — jedes Mal `get_bitsize() == 8`.
  Wer aus einem `Uint32`-Feld eine 32-Bit-Fläche erwartet, irrt.
* **`surface_to_array(.., "A")` gibt 255, sobald die Fläche nicht
  mischt.** Nach `set_alpha(None)` ist eine Fläche undurchsichtig — auch
  mit Alphakanal. `surfarray_test` sagt es im Fehlertext: *„All alpha
  values should be 255 when surf.set_alpha(None) has been set."*

### Sieben Befunde aus dem Zug davor

* **`PixelArray` braucht Start und Schritt, nicht zwei Indexlisten.** Mit
  Listen ist `strides` nicht mehr ablesbar (pygame meldet nach `[::2, :]`
  den doppelten Schritt), `transpose()` einer eindimensionalen Sicht ist
  nicht darstellbar, und `_pixels_address` hängt am Anfangspixel, nicht
  an der Menge der getroffenen Pixel.
* **Eine Sperre muss ein weicher Verweis sein.** Hält die Fläche ihr
  `PixelArray` fest, stirbt es nie — und `s.get_locked()` bleibt für
  immer wahr. Und `unlock()` löst die **eigene** Sperre, nicht die
  letzte: ein blankes `pop()` nahm einem offenen `BufferProxy` die seine
  weg.
* **PyO3 0.26 hat keinen `tp_init`-Schlitz.** Eine Methode namens
  `__init__` steht im Wörterbuch des Typs, aber `type.__call__` ruft sie
  nie. Daraus folgten drei Umwege — und einer davon hat eine **benannte
  Grenze aufgelöst**: `PixelArray` und `mixer.Sound` sind jetzt dünne
  Python-Unterklassen, `Window.__init__` **stellt nach**, statt
  anzulegen.
* **Bei `BLENDMODE_NONE` mischt SDL nicht mit dem Alpha der Eckfarben.**
  Ein texturiertes Dreieck mit der Eckfarbe `(100,100,100,100)` gibt
  genau `Textur × 100/255` — das vierte Byte bleibt liegen.
* **`pygame.freetype` legt mit geradem Alpha übereinander.** `fgcolor =
  (10,20,30,128)` auf durchsichtigem Grund gibt dort genau
  `(10,20,30,128)`; SDLs Mischung gäbe `(5,10,15,128)`.
* **`STYLE_WIDE` ist dieselbe Maschine wie `STYLE_STRONG`.** Beide
  verschmieren das Bild nach rechts; fett nimmt `ptsize / 10`, breit
  `ptsize × strength`. Gemessen bei Größe 64: fett +6 Spalten je Zeichen,
  breit +2. Damit war `strength` kein leeres Feld mehr, und `ftfont`
  ließ sich überhaupt erst bauen — es setzt den Wert bei jeder Schrift.
* **Eine Größe ist bei `load_sized_svg` ein Rahmen, kein Auftrag.** Bei
  `size=(1000, 30)` und einem quadratischen Dokument kommt eine Fläche
  **30 × 30** heraus: das Seitenverhältnis bleibt, eine Seite trifft den
  Rahmen genau.

### Was noch fehlt

**Kein Test von 2485 weicht mehr ab.** Hier stand erst „vier Tests",
dann „zwei" — und **keiner der vier war das, was der Vermerk daneben
behauptete.** Alle vier standen als *Rasterkanten* da („FreeType rastet
die Kontur aufs Pixelgitter ein, fontdue nicht; an einer Ecke steht dann
eine Deckung von 1 statt 0"). Das ist eine echte Grenze dieses Projekts,
und sie war hier viermal die falsche Erklärung:

| Test | der Vermerk | was wirklich fehlte |
|---|---|---|
| `freetype.render` | Rasterkante | das Referenzbild `A_PyGameMono-8.png` ist aus dem **eingebetteten Bitmapschnitt** exportiert, und den gab es hier nicht |
| `freetype.use_bitmap_strikes` | eingebettete Schnitte fehlen | **richtig** — und es war derselbe Mangel wie in der Zeile darüber |
| `ftfont.issue_font_alphablit` | Rasterkante an der Ecke | `pygame.freetype` lud die Standardschrift **zu klein**: der Faktor 0,6875 aus `font.c` gehört zu `pygame.font`, nicht dorthin. Das Bild war ein Drittel zu klein, und bei `ftfont` wirkte der Faktor sogar **doppelt**. |
| `freetype.render_to_mono` | Rasterkante an der unteren linken Ecke | **drei** Dinge: der Kasten eines Leerzeichens, das verschluckte Alpha der Schriftfarbe, und `render_to` in **einer** statt zwei Stufen |

Die Lehre ist dieselbe wie überall in diesem Projekt, nur diesmal
besonders teuer: **ein Vermerk, der eine bekannte Grenze nennt, ist der
bequemste Platz, um einen Fehler zu verstecken.** „Rasterkante" war
plausibel, nicht nachgemessen, und hat vier Fehler vier Tage gedeckt.

Die Einzelheiten stehen unten unter „Eingebettete Bitmapschnitte", „Zwei
Textkisten, zwei Größen" und „Zwei Stufen, nicht eine".

### Was dazugekommen ist: die Dateiformate

`transform` ist damit **vollständig**, und `image` sowie `mixer_music`
haben den größten Sprung gemacht. Elf Formate mehr, und die Regel dahinter
war jedes Mal dieselbe: **erst messen, was echtes pygame aus der Datei
macht, dann bauen.** Tiefe und Flaggen sind nämlich nicht ableitbar.

| Format | Weg | pygame liefert |
|---|---|---|
| JPEG | `zune-jpeg` / `jpeg-encoder` | 24 Bit, Flaggen 0 |
| GIF | `gif` (LZW über `weezl`) | 8 Bit mit Tafel 256; durchsichtiger Index → **Schlüsselfarbe** |
| WebP | `image-webp` (VP8 und VP8L) | 24 Bit ohne Alpha, 32 mit |
| TIFF | `tiff` | **immer** 32 Bit mit `SRCALPHA` |
| TGA | von Hand, lesen **und** schreiben | 8/15/24/32 Bit, je nach Kopf |
| PNM (PBM, PGM, PPM) | von Hand, `P1` bis `P6` | 8 Bit mit Tafel 2 bzw. 256, sonst 24 |
| QOI | von Hand | **immer** 32 Bit mit `SRCALPHA` |
| PCX | von Hand, mit Bitebenen | 24 Bit bei drei Ebenen, sonst Tafel |
| XPM | von Hand — es ist **C-Quelltext** | 8 Bit, Tafel so lang wie die Farbliste |
| LBM | von Hand, IFF-ILBM mit Bitebenen | 8 Bit, Tafel aus `CMAP` |
| FLAC | `rodio`-Merkmal `flac` (claxon) | byteweise wie SDL_mixer |
| MP3 | `rodio`-Merkmal `mp3` (symphonia) | byteweise wie SDL_mixer |
| MS-ADPCM | von Hand, in `wav.rs` | byteweise wie SDL_mixer |
| IMA-ADPCM | von Hand, in `wav.rs` | (keine Beispieldatei zum Messen) |
| SVG | `resvg` / `usvg` | Fläche mit `SRCALPHA`; Größe ist ein **Rahmen** |
| XCF | `xcf-rs`, Ebenen hier zusammengelegt | das zusammengerechnete Bild |
| TIFF mit Farbtafel | von Hand, LZW über `weezl` | die `tiff`-Kiste lehnt `RGBPalette` ab |
| OPUS | `opus-pure` (samt Ogg) | **immer 48 kHz** — der Kopf nennt nur, was hineinging |
| WavPack | `symphonia-codec-wavpack` | verlustfrei; eigener Behälter aus `wvpk`-Blöcken |
| XM | `xmrs` + `xmrsplayer` | ein Notenblatt, kein Tonstrom — es wird **gespielt** |

Alle zwanzig sind **reines Rust**. Die sechs untersten kamen zuletzt dazu und haben
`image` auf 50 von 50 und `mixer.music` auf 30 von 30 gebracht; für OPUS,
WavPack und XM stand hier vorher, es gebe „in reinem Rust keinen Dekoder
in Reichweite". Das war 2025 wahr und ist es nicht mehr — eine Grenze,
die an der Lage der Kisten hängt, gehört nachgesehen und nicht
abgeschrieben. Die fünf kleinen Formate stehen von Hand da, weil sie
zusammen kürzer sind als die Abhängigkeit, die sie ersparen würden, und
weil sie die Kontrolle über Tiefe und Flaggen lassen, auf die es hier
ankommt.

Drei Befunde, die ein Nachbau nicht errät:

* **`image.load_animation` gibt Millisekunden, nicht Sekunden.**
  `animated_sample.gif` liefert zehnmal `150.0`, und in der Datei stehen
  15 Hundertstelsekunden. Sekunden wären die naheliegende Wahl gewesen und
  um den Faktor 1000 daneben.
* **Ein TGA mit 16 Bit meldet sich als 15**, und das eine Alphabit von
  A1R5G5B5 wird übergangen — auch wenn es gesetzt ist.
* **`metadata.mp3` hat eine falsche ID3-Längenangabe.** Der Kopf nennt
  198 Bytes, die Rahmen sind 202 lang, der Ton fängt bei 212 an. SDL_mixer
  nimmt es hin, symphonia nicht (`out of bounds`). Deshalb wird der Block
  hier selbst gelesen — was zugleich `music.get_metadata()` füllt.
* **Bei PBM heißt das Bit `0` weiß**, nicht schwarz, und pygames Tafel
  steht genau so herum.
* **`image.save` in ein Dateiobjekt ohne Hinweis schreibt TGA**, nicht
  PNG — `image.c:182` sagt `name = (namehint ? namehint : "tga")`.

### Eine gleiche Länge ist kein gleicher Inhalt

ADPCM war fertig: `secosmic_lo.wav` gab 576.000 Bytes und 3,2653 Sekunden,
genau wie pygame. Erst der SHA-256 zeigte, dass die **Werte** abweichen —
und zwar bei *jeder* Datei, auch bei reinem PCM. Der Dekodierer war nicht
schuld; es war der **Umrechner** der Abtastrate, den nie jemand gemessen
hatte.

```text
house_lo.wav, 22050 → 44100, Mono → Stereo
rustgame (linear): -3584, -3712, -3840, -3968
pygame   (SDL):    -3578, -4150, -4388, -4339
gleich: 10.602 von 626.648 Werten (1,7 %)
```

Kein Test in pygames Suite prüft Abtastwerte nach dem Umrechnen. Die
Abweichung wäre also unbemerkt geblieben — gefunden wurde sie nur, weil
die Byte-Zahl stimmte und der Hash nicht.

Behoben durch Übertragung von `SDL_ResampleAudio` aus `SDL_audiocvt.c`
samt Filtertabelle: ein fensterbegrenzter Sinc mit fünf Nulldurchgängen je
Flügel und 512 Stützstellen dazwischen. Drei Dinge daran sind nicht
erratbar:

* **Die Reihenfolge.** `SDL_BuildAudioCVT` hängt *erst* die Spuren um und
  filtert *danach*. Wer Stereo auf Mono mischt, muss das vor dem Filter
  tun — sonst filtert er zwei Spuren einzeln und mittelt hinterher.
* **Die Ränder sind Stille**, nicht der erste oder letzte Wert.
* **Die beiden Zahlenumwandlungen sind Bit-Kniffe.** SDL rechnet nicht
  `wert / 32768.0`, sondern baut die Gleitkommazahl direkt zusammen
  (`x.u32 = (Uint16)src ^ 0x43808000; dst = x.f32 - 257.0f`). Das ist kein
  Schönheitsfehler, den man begradigen darf: die beiden Wege runden
  verschieden, und die Differenz wandert durch den Filter in jeden Wert.

Die Tabelle wird **erzeugt**, nicht abgetippt —
`tools/gen_resampler_filter.py` liest SDLs Header und schreibt
`filter.rs`. Sie nachzurechnen hieße, Kaiser-Fenster und Bessel-Funktion
bitgenau zu treffen; jede Abweichung im letzten Bit multipliziert
hinterher jeden Abtastwert.

Ergebnis: **WAV, ADPCM und FLAC stimmen jetzt Byte für Byte mit pygame
überein** — derselbe SHA-256 über den ganzen umgerechneten Klang. Übrig
bleiben OGG und MP3, und dort ist es der Dekodierer, nicht die Kette: im
Mittel eine Eins im letzten Bit.

### Was sich dabei über die eigenen Tabellen gezeigt hat

Die API-Zahl ist von 598 auf 608 gestiegen, **ohne dass eine Zeile
Engine dazukam**: in `tools/api_coverage.py` standen zwölf handgepflegte
Ausnahmen, und alle zwölf waren veraltet.

```text
width, height, size                   "-> get_width()" usw.   -> gibt es
get_palette, set_palette, ...         "keine Palettenflaechen" -> gibt es
set_masks, set_shifts                 "Format liegt fest"      -> sagt
                                      wortgleich dasselbe wie pygame
image.load_animation                  "braeuchte einen Dekodierer" -> da
system.get_power_state/_pref_locales  "braeuchte FFI"          -> antworten
```

Jeder Satz stimmte, als er geschrieben wurde. **Eine Tabelle, die
Ausnahmen von Hand pflegt, veraltet genauso wie ein Vermerk im
Quelltext** — und sie veraltet unauffälliger, weil sie die Zahl, mit der
man den Fortschritt misst, selbst erzeugt.

### Die dritte Sorte Messfehler: das Messgerät selbst

Die API-Zahl ist zweimal *gefallen*, ohne dass etwas kaputtging — beide
Male war die Messung besser geworden, nicht die Engine schlechter. Bei der
Testsuite ist dasselbe passiert, nur schärfer: sie ist von 1261
**sichtbaren** Tests auf 1765 gestiegen. Fast fünfhundert Prüfungen waren
nie gelaufen, und zwar nicht rot, sondern **unsichtbar**.

Sechs Rätsel, die wie sechs verschiedene Probleme aussahen, hatten je
eine Ursache im Läufer:

| Symptom | Ursache |
|---|---|
| `font` bringt den ganzen Lauf zum Stehen | `font_test.py:1104` trägt `__tags__ = ["interactive"]`. Die Klasse fragt auf der Konsole nach und **wartet**. |
| `event` bricht stumm ab | Ein einzelner Test hängt: der vorhergehende sperrt mit `set_blocked(None)` alle Ereignisse und nimmt es nie zurück, danach wartet `wait()` auf eines, das nie kommt. **Echtes pygame hängt dort genauso** — nachgemessen. |
| `scrap` meldet **−2** gute Tests | Die ganze Datei trägt `("ignore", "subprocess_ignore")`; pygame lässt sie nie laufen. |
| `image` meldet **−3** gute Tests | `unittest` zählt *Einträge*, nicht Tests: ein `subTest` erzeugt je Durchlauf einen Fehlereintrag, `testsRun` zählt einmal. 53 Einträge auf 50 Läufe. |
| `transform` verliert drei Tests an fremden Flächen | `test_utils` wurde **vor** dem Umbiegen geladen, also erbte `SurfaceSubclass` vom echten pygame. |
| `joystick` lädt gar nicht | Es schreibt `import pygame._sdl2.controller` — einen **zweistufigen** Namen, und der Läufer bog nur die erste Ebene um. Eine naive Rekursion lief dabei in einen Ring: die Module verweisen aufeinander (`mixer.music is mixer_music`), 994 Wiederholungen bis zum `RecursionError`. Jetzt genau zwei Ebenen. |

Der Läufer wendet jetzt pygames **eigene** Ausschlussregeln an
(`run_tests.py`: `interactive`, `ignore`, `python3_ignore`, `SDL2_ignore`),
zählt nach eindeutigen Tests statt nach Einträgen, und lädt das
Hilfswerkzeug nach dem Umbiegen neu.

Eine negative Zahl ist harmlos, solange man sie sieht. Gefährlich war sie,
weil sie **jede andere Zahl derselben Zeile mitverschob**: `image` sah aus
wie „52 Fehler von 50 Tests", und der wahre Befund stand nirgends.

**Ein Ausschluss darf nicht mehr treffen, als er soll.** Der hängende
`event`-Test trägt die Marke `timing`, und die pauschal auszuschließen war
verlockend und falsch: `time_test.py` markiert seine beiden Klassen
ebenfalls damit, und damit fielen 14 einwandfreie Tests weg, von denen 11
grün waren. Jetzt steht dort ein **Name** statt einer Marke — eine Zeile
mehr, und sie sagt genau, was gemeint ist.

### Wie die API-Zahl ehrlich wurde

`API.md` wird erzeugt, nicht gepflegt. Die pygame-Seite kam schon immer
aus `dir()` am **installierten pygame**, konnte also nicht veralten. Die
rustgame-Seite dagegen war ein Regenwurm aus regulären Ausdrücken über
die `.rs`-Dateien — und der beantwortete die Frage *„steht dieses Wort
irgendwo im Quelltext"*. Die Frage lautet aber *„kommt ein
Python-Programm daran"*, denn das Ziel heißt `import rustgame_py as
pygame`.

| Messung | Zahl |
|---|---:|
| Quelltext durchsucht | 99 % |
| **gebaute Hülle** per `dir()` gefragt | 83 % |
| + zehn Module, die die Tabelle nie abfragte | 76 % |
| nach der Arbeit daran | 97 % |
| + acht Namen, die die Tabelle **versteckt** hatte | 98 % |
| nach der Arbeit daran | **99 %** |

Die vorletzte Zeile ist der dritte Fall derselben Sorte, und der
unangenehmste. In der Ausnahmeliste des Messgeräts stand:

```python
EGAL = {
    "get_view", "get_buffer", "get_locked", "lock", "unlock",
    "mustlock", "get_locks", "premul_alpha_ip", ...
```

mit der Begründung, das seien „Dinge, die es in Rust nicht geben kann
(Puffer-Protokoll, C-API, numpy-Brücken)". **Alle acht gibt es, und alle
acht tun ihre Arbeit.** `Surface` stand mit 48 von 48 in der Tabelle,
obwohl es 56 von 56 waren: eine Frage, die nicht gestellt wird, kann nicht
mit „fehlt" beantwortet werden — aber auch nicht mit „da".

Was der alte Weg verdeckt hatte:

- `pygame.sprite` und `pygame.mixer` standen mit **19 von 19** in der
  Tabelle, weil die Rust-Kisten fertig sind. In der Python-Hülle gab es
  die beiden Module **überhaupt nicht**.
- `Vector2.angle` galt als vorhanden, weil der Kern ein `pub fn angle`
  hat. `rustgame_py.math.Vector2(3,4).angle` gab es nicht.
- `Rect.move` galt als vorhanden, weil im Quelltext `move_` steht (`move`
  ist in Rust ein Schlüsselwort). In Python hieß die Methode dann auch
  `move_`, und jedes `r.move(1, 2)` scheiterte.
- `pygame.cursors`, `joystick`, `scrap`, `system`, `version`,
  `pixelarray` und vier weitere Module standen gar nicht erst in der
  Liste. Was nicht abgefragt wird, kann auch nicht fehlen.

Die Tabelle kennt jetzt **fünf** Kategorien:

- **Da** — der Name ist aus Python erreichbar und tut etwas.
- **Fehlt** — es gibt ihn nicht.
- **Nur im Kern** — die Rust-Kiste hat ihn, die Hülle reicht ihn nicht
  durch. **Zählt als fehlend**, denn aus Python kommt man nicht daran.
- **Anders gebaut** — er existiert in anderer Form und zählt weder als
  das eine noch als das andere (`Rect.x` ist ein öffentliches Feld,
  `MaskType` ist in pygame selbst nur ein Zweitname für `Mask`).
- **Nur als klarer Fehler** — der Name existiert, meldet aber bloß, dass
  es das hier nicht gibt. **Zählt als fehlend.** Die Funktion existiert
  trotzdem, weil ein Programm sonst gar nicht erst übersetzt und man
  raten müsste, woran es liegt.

### Was vollständig ist

`Surface`, `Rect`, `FRect`, `Color`, `draw`, `transform`, `time`, `font`,
`image`, `display`, `event`, `key`, `mouse`, `mask`, `math`, `mixer`,
`mixer.music`, `sprite`, `cursors`, `joystick`, `scrap`, `version`,
`pixelarray`, `bufferproxy`, `base`, `pixelcopy`, `geometry`, `gfxdraw`,
`freetype`, `midi`, `rwobject`, `typing`, `sysfont`, `ftfont`, `system`,
`_render`, `surfarray`, `sndarray` — dazu die Klassen `Vector2`,
`Vector3`, `Mask`, `Clock`, `Font` (beide), `Sound`, `Channel`, `Sprite`,
`Group`, `Circle`, `Line`, `PixelArray`, `BufferProxy`, `Renderer`,
`Texture` und `Image`, jede einzeln geprüft.

**Offen ist kein Name mehr.** Die Liste war einmal sechzehn lang, und
jeder Eintrag ist einzeln gefallen — keiner davon, weil später doch noch
etwas gebaut wurde, das zu schwer schien. Sie fielen, weil der **Vermerk
daneben** etwas anderes gemessen hatte als das, was er behauptete.

Die letzten drei, alle am selben Tag und mit demselben Satz:

| | der Vermerk | was ihn umwarf |
|---|---|---|
| `camera.Camera` | „ein Kamerabild braucht den Gerätezugriff des Betriebssystems (DirectShow, V4L2), und der ginge nur über FFI" | die Kiste `nokhwa` — Media Foundation, V4L2, AVFoundation hinter einer sicheren Schnittstelle |
| `midi.Input` | „bräuchte PortMidi oder einen Ersatz" | die Kiste `midir` — WinMM, ALSA, CoreMIDI, ebenso |
| `midi.Output` | dito | dito |

Der erste Halbsatz war jedes Mal wahr: es **braucht** den Gerätezugriff
des Betriebssystems. Der zweite war jedes Mal dieselbe Verwechslung:
**`unsafe` braucht irgendwer, aber nicht dieses Programm.** Vorher war
`system.get_total_ram` mit genau diesem Satz gefallen (`sysinfo`), und
das war die erste von vier.

Was die Begründung jedes Mal zum Einsturz brachte, war keine neue
Überlegung, sondern **eine Suche nach einer Kiste**. Die Notiz hatte nie
danach gesucht; sie hatte aus „ich müsste FFI schreiben" geschlossen, dass
niemand es schon geschrieben hat.

Weggefallen sind außerdem:

* **Die sieben von `window.Window`** — `flash`, `set_modal_for`, `opacity`
  (Setzer), `opengl`, `maximize`, `minimize`, `restore`. Der Vermerk
  lautete: minifb habe für keines einen Aufruf. Das stimmte für minifb und
  war die falsche Frage. **Gemessen unter demselben Treiber, unter dem
  alles andere gemessen wird**, antworten alle acht Proben gleich — und
  **pygame selbst verweigert drei von ihnen** unter `dummy`.

* **Die vier von `pixelcopy`.** Hier stand: „arbeitet auf
  mehrdimensionalen Feldern; was geht, ist `numpy.ndarray`, und numpy ist
  hier nicht installiert. Ohne Messung wird in diesem Projekt nichts
  gebaut." Beide Hälften waren falsch geworden. Das Puffer-Protokoll gibt
  es (siehe unten), und zum **Messen** braucht es kein numpy: pygames
  eigenes `tests/test_utils/buftools.py` baut ein Feld mit jeder Form,
  Schrittweite und Byteordnung, und `arrinter.py` eines mit rohem
  `__array_struct__`. Beide lagen die ganze Zeit im selben Ordner wie die
  Tests, die sie benutzen.
* **`system.get_total_ram`.** Hier stand: „bräuchte einen Systemaufruf
  über FFI, und im ganzen Workspace steht kein einziges `unsafe`." Das ist
  eine Verwechslung: `unsafe` braucht **irgendwer**, aber nicht dieses
  Programm. Die Kiste `sysinfo` tut es hinter einer sicheren
  Schnittstelle, und die Zahl kommt jetzt in Megabyte heraus, wie SDLs
  `SDL_GetSystemRAM` sie meldet.
* **`midi.time`** war einfach vergessen — PortMidis Uhr, hier der
  Millisekundenzähler ab `midi.init()`.
* **`_sdl2.video.get_grabbed_window`** stand als „nur im Kern" da, weil
  die Python-Seite auf `display.get_grabbed_window` zeigte und es dort
  nicht gibt. **Ein Zeiger auf eine leere Stelle ist schlimmer als kein
  Zeiger**: er sieht aus wie eine Umsetzung.

### Die Zahl im Plan, die nie eingelöst wurde

Im Plan steht als **Endabnahme**: „PyGameZ zweimal starten — einmal mit
echtem pygame, einmal mit `rustgame` — und die PPM-Bytes aus `_present()`
für die ersten **300** Bilder vergleichen."

Gefahren wurden **dreißig**. Nicht aus Nachlässigkeit, sondern weil
dreißig genügen, um jede Zeichenart einmal zu durchlaufen — und weil
niemand die Zahl nachgehalten hat. Mit 300 fiel sofort etwas auf:

```text
30 Bilder    alles gleich
300 Bilder   ereignisse  ABWEICHUNG
```

In der Spur stand `AUDIODEVICEADDED`: pygame hatte **sechs** dieser
Ereignisse in der Schlange, rustgame keines. Über dreißig Bilder waren sie
noch nicht alle abgeholt, über dreihundert schon.

**Eine Zahl im Plan, die nie eingelöst wurde, ist dasselbe wie ein
Vermerk, den niemand nachhält.** `vergleich_spiel.py` nimmt die Zahl jetzt
als Argument; bei 300 und bei 1000 ist alles gleich.

#### Und was dahinter lag: `pygame._sdl2` hat acht Untermodule

Der Faden führte zum **achten blinden Fleck**. `pygame._sdl2` hat acht
Untermodule; `api_coverage.py` fragte nach **einem** (`_sdl2.video`), und
rustgame hatte drei:

| | pygame | vorher hier |
|---|---|---|
| `audio` | 18 Konstanten, `AudioDevice`, `get_audio_device_names` | **fehlte** |
| `controller_old` | die alte Fassung, vier Namen mehr als `controller` | **fehlte** |
| `mixer` | `set_post_mix` | **fehlte** |
| `sdl2` | neun `INIT_*`-Flaggen, `init_subsystem` | **fehlte** |
| `window` | `Window` | **fehlte** |
| `controller`, `touch`, `video` | | ✓ |

Alle fünf sind jetzt da, und die Messung hat beim Bauen fünf weitere
Einzelheiten geliefert, die man nicht erraten kann:

* **`cpal` sieht genau dieselben Tongeräte wie SDL** — fünf Ausgabe-, ein
  Aufnahmegerät, gleiche Namen, gleiche Reihenfolge. Der einzige
  Unterschied steht in SDLs eigener Beschreibung: *„Returns a list of
  **unique** devicenames"*. Zwei Geräte dieser Maschine heißen gleich, und
  SDL hängt dem zweiten ` (2)` an.
* **`get_audio_device_names()` ohne Ton ist ein Fehler** —
  `error: Audio system not initialised`, auch nach `mixer.quit()`. Bei
  rustgame hängt die Liste technisch *nicht* am Mixer; sie wird trotzdem
  verweigert, weil ein Programm, das sich darauf verlässt, bei pygame
  scheitert.
* **`INIT_EVERYTHING` ist nicht die Summe der anderen.** 62001 ist
  `0xF231`, und darin fehlt `INIT_NOPARACHUTE`. Wer die Zahl herleitet
  statt sie abzulesen, liegt daneben.
* **`controller_old.Controller` ist eine *eigene* Klasse**, nicht ein
  Zweitname von `controller.Controller` — gemessen ist
  `controller_old.Controller is controller.Controller` **falsch**. Und
  `Controller(True)` gibt dort denselben Fehler wie `Controller(1)`, keinen
  `TypeError`: ein `bool` ist an dieser Stelle eine Zahl.
* **`_sdl2.video` hatte bei uns einen Namen zu viel** (`messagebox`), den
  pygame nicht hat. Ein Name zuviel ist derselbe Fehler wie einer zuwenig
  — beide sagen etwas Falsches über die Schnittstelle. Er heißt jetzt
  `_messagebox`.

Was **nicht** geht, steht als benannte Grenze da: `AudioDevice` und
`set_post_mix` brauchen einen rohen Tonstrom mit Rückruf aus dem Tonfaden,
und rustgames Mixer hat gar keine zentrale Mischstufe. Nebenbei gemessen:
pygame nimmt `set_post_mix(7)` an und **stürzt ab**, sobald der Haken
gerufen wird — kein `TypeError`, ein Speicherzugriffsfehler in
`recording_cb`. Das ist der Grund, warum diese Grenze keine schmerzt.

### Zwei Namen, die zwei Messgeräte zugleich übersehen haben

`pygame.print_debug_info` schreibt die Auskunft, die man an eine
Fehlermeldung hängt — Plattform, Python, SDL-Versionen, Treiber. Es ist
reines Python (186 Zeilen in `pygame/_debug.py`) und darum **wörtlich
übertragen**, wie `sprite.py`, `sysfont.py`, `colordict.py` und
`cursors.py`. Geändert sind nur die Importzeilen.

```text
Platform:		Windows-11-10.0.26200-SP0
Processor:		AMD64 …	SSE2: Yes	AVX2: Yes	NEON: No
Python:			CPython 3.14.4 …
pygame version:		2.5.7
SDL versions:		Linked: 2.32.10	Compiled: 2.32.10
SDL Mixer versions:	Linked: 2.8.1	Compiled: 2.8.1
…
```

Siebzehn Zeilen, dieselben wie bei pygame. Drei der Zahlen sind ehrlich
geborgt: rustgame bindet weder SDL_mixer noch SDL_ttf noch SDL_image ein,
und die Versionen stehen trotzdem da — weil `image_test.py` sie als
**Paar** mit `get_extended()` prüft. Siehe „Versionsnummern" weiter unten.

`pygame.docs` sagt, ob die erzeugte Doku mitgeliefert wurde. Hier
**nein**, und das ist die Aufgabe des Moduls und kein Mangel: pygame hat
es gebaut, um genau diese Frage beantworten zu können.

**Beide Namen fehlten, und beide Messgeräte haben es zugleich übersehen** —
auf zwei verschiedene Weisen:

```text
pygame_tests.py   47 Namen in einer abgetippten Liste, 50 Dateien im Ordner
                  nie gefahren: debug_test.py, docs_test.py,
                                image__save_gl_surface_test.py
api_coverage.py   54 Einträge für Untermodule und Klassen,
                  aber keiner für `dir(pygame)` selbst
```

Die beiden Lücken deckten sich gegenseitig: `debug_test.py` prüft
`print_debug_info`, `docs_test.py` prüft `docs` — und beide Namen hätte
auch die API-Tabelle melden müssen. Darüber stand im Bericht der Satz „die
Testdateien werden aus dem Ordner gelesen statt aufgezählt", und er stimmte
einfach nicht.

Jetzt liest `alle_dateien()` den Ordner, `REIHENFOLGE` gibt nur noch die
Lesefolge an, und `MODULE` hat einen Eintrag `""` für das Hauptmodul. Beim
Schärfen dieser zweiten Frage fiel noch eine Falle auf, die zum Muster
gehört: die erste Fassung zählte auch `types`, `difflib`, `itertools` und
`atexit` als fehlende Namen, weil `sprite.py` und `sysfont.py` sie
importieren und sie danach in `dir()` stehen. **Ein Netz, das weiter
wirft, fängt auch, was kein Fisch ist.** Gezählt wird jetzt nur, was
seinen eigenen Namen am Weg dorthin hat (`pygame.surflock`, nicht
`pygame.sprite.types`).

### Flächen dürfen den Faden wechseln

Hier stand im Plan eine Zeile, die sich später bezahlt hat:

> **Surface-Speicher:** `Rc<RefCell<PixelBuf>>` + Offset, hinter einem Alias
> `type Shared<T>`, damit später `Arc<Mutex<..>>` eine Zeile kostet.

Fällig wurde sie durch einen Fehler, der **würfelte**.
`image_test.test_load_bmp_threads` lädt Bilder in zwanzig Fäden und gibt
sie an den Hauptfaden weiter. Gebaut wurden sie alle; beim **Wegräumen**
kam

```text
RuntimeError: PySurface is unsendable,
              but is being dropped on another thread
```

— mal zwei Fehler im Lauf, mal keiner, je nachdem, welcher Faden gerade
aufräumte. **Ein Fehler, der würfelt, ist schlimmer als einer, der immer
kommt**: er verschwindet beim Nachsehen.

Die Zeile kostete dann nicht eine Zeile, sondern drei Aliase und knapp
siebzig Stellen — der Plan hatte nur den Kern gesehen, und `Rc<RefCell<..>>`
lag an **drei** Stellen:

| | vorher | jetzt |
|---|---|---|
| `rustgame-core`, Pixelspeicher | `Rc<RefCell<PixelBuf>>` | `Arc<ReentrantMutex<RefCell<..>>>` |
| `rustgame-py`, die Fläche | `Rc<RefCell<Surface>>` | `Arc<RwLock<Surface>>` |
| `rustgame-display`, der Schirm | `Rc<RefCell<Surface>>` | `Arc<RwLock<Surface>>` |

#### Drei verschiedene Sperren, und jede aus einem Grund

**Der Kern sperrt verschachtelt.** Ein Blit hält Quelle und Ziel; bei
einem Selbstblit ist das **dieselbe** Sperre. Ein `Mutex` hängt dort für
immer, ein `RefCell` bricht ab — und ein Abbruch ist eine Fehlermeldung,
ein Hänger ein Programm, das steht. `ReentrantMutex<RefCell<T>>` lässt
denselben Faden wieder hinein und behält dabei RefCells Abbruch bei zwei
gleichzeitigen Schreibleihen. Genau die Semantik von vorher, nur über
Fadengrenzen.

**Die Hülle kann das nicht.** Sie muss einen Wächter **herausgeben**, und
zwar einen, dessen Lebensdauer an `&self` hängt — `PySurface::leihen()`
wird an 94 Stellen gerufen. Bei einer Stufe geht das. Bei zwei Stufen
leiht der innere `Ref` vom **äußeren Wächter** und nicht von `&self`, und
der ist beim Verlassen der Funktion weg; zwei Stufen brauchen einen
selbstbezüglichen Typ, und den gibt es in sicherem Rust nicht. Also eine
Stufe — und `RwLock` statt `Mutex`, weil `RefCell` **viele Leser** erlaubt
und ein `Mutex` daraus einen Fehler gemacht hätte, den es vorher nicht
gab.

**Gesperrt wird nur mit `try_`.** `try_read` und `try_write` scheitern
genau dort, wo `try_borrow` und `try_borrow_mut` gescheitert sind, und
geben denselben Fehler, den pygame dort gibt: `Surfaces must not be
locked during blit`. Beim Schirm bleibt damit auch die dokumentierte
Panik erhalten — wer während `flip()` noch eine Leihe hält, bekommt sie
weiterhin, nur mit einer Meldung, die sagt, worum es geht.

Was es gebracht hat: **PyO3s `unsendable` ist von `Surface` weg.** Vier
`Cell<bool>` wurden zu `AtomicBool` und eine `RefCell<Vec<..>>` zu einem
`Mutex<Vec<..>>` — PyO3 0.26 verlangt für eine `#[pyclass]` ohne
`unsendable` nicht nur `Send`, sondern **`Send + Sync`**. Danach: 120
Läufe mit zwanzig Fäden, keine Meldung, und zwölf vollständige Durchgänge
der Testsuite hintereinander mit `2364 / 0 Abweichungen / 0 Fehler`.

### Der Fehler, der würfelte — und wo er wirklich saß

Mehrere Durchgänge zeigten `2362 / 0 Abweichungen / **2 Fehler**`, andere
`2364 / 0 / 0`. Derselbe Stand, dieselbe Maschine. Ich habe drei
Vermutungen geprüft und alle drei widerlegt — Systemlast, ein `cargo test`
davor, Ausgabe durch eine Pipe statt in eine Datei. Erst der vierte
Versuch, den ganzen Durchgang **mit Fehlerspur in eine Datei** zu
schreiben, hat ihn eingefangen:

```text
ftfont              61       0       2       9
  FEHLER test_load_default_font_filename
         FileNotFoundError: Das System kann die angegebene Datei nicht finden.
           f = pygame_font.Font(pygame_font.get_default_font(), 20)
```

Nicht der Fadenlader also, sondern **`ftfont`** — und allein gefahren war
die Datei immer grün. Zwei Dinge trafen zusammen:

* **`pygame.freetype.Font("freesansbold.ttf")` scheitert**, und zwar bei
  pygame genauso (gemessen: `No file 'freesansbold.ttf' found in working
  directory`). `ftfont` fängt das ab: der mitgelieferte **Name** steht dort
  für die mitgelieferte Schrift, `Font("freesansbold.ttf")` ist dasselbe
  wie `Font(None)`. Diese Umschreibung steht bei pygame in `__init__` —
  **und bei PyO3 ist das zu spät.** `#[new]` lädt die Datei schon, mit den
  Argumenten, die der Aufrufer gegeben hat; es gibt keinen
  `tp_init`-Schlitz. Jetzt steht die Umschreibung in `__new__`.
* **Und die Suche nach der Schrift hing am Arbeitsverzeichnis.** Drei ihrer
  vier Stellen tun das, und ein früher gelaufener Test wechselt es — daher
  das Würfeln: lag `freesansbold.ttf` zufällig vom Arbeitsverzeichnis aus
  erreichbar, ging es gut. Die Schrift liegt mitgeliefert **neben der
  Hülle**, und dieser Ordner ändert sich nicht; die Suche fragt ihn jetzt
  zuerst.

Der naheliegende Weg, diesen Ordner zu erfahren, war `m.filename()` in der
Modul-Einrichtung. **Der geht nicht:** `__file__` setzt die
Einfuhrmaschinerie erst, *nachdem* die Einrichtungsfunktion zurückgekehrt
ist — die Suche fiel danach weiterhin auf das Arbeitsverzeichnis zurück.
Geholt wird er jetzt beim **ersten** Schriftzugriff, wo Python läuft und
`__file__` steht.

**Die Lehre:** ein Fehler, der von der Reihenfolge abhängt, sieht aus wie
Zufall. Und man fängt ihn nicht durch Nachdenken, sondern indem man den
ganzen Lauf mit Spur in eine Datei schreibt und wartet, bis er wieder
kommt.

### Kamera und MIDI — echte Geräte

Die drei letzten offenen Namen waren alle Gerätezugriff, und darum fielen
sie zusammen. Was jetzt läuft, ist gemessen und nicht behauptet:

```text
rustgame.camera.list_cameras()            ['USB  Live camera']
rustgame.camera.Camera(..).get_image()    (320, 240), 32 Bit, nicht schwarz
rustgame.midi.get_count()                 1
rustgame.midi.Output(0).note_on(60, 100)  klingt
```

Die Aufteilung ist dieselbe wie bei pygame, und zwar aus demselben Grund.
pygames `camera` und `midi` sind je zur Hälfte Python und zur Hälfte C:
oben die Zählarbeit, unten das Gerät.

| | wo es liegt | was dort steht |
|---|---|---|
| Kamera, Gerät | `rustgame-display/src/kamera.rs` | Geräteliste, Formatwahl, Bild holen, Spiegeln |
| Kamera, Rechnerei | `rustgame-py/src/python/camera.py` | `colorspace`, `AbstractCamera`, die Regler |
| MIDI, Gerät | `rustgame-mixer/src/midi.rs` | Geräteliste, Ein- und Ausgang, Posteingang |
| MIDI, Rechnerei | das eingebettete `pygame.midi` in `lib.rs` | Notennummern, Kanäle, `midis2events` |

Die Kamera liegt bei `rustgame-display`, weil Geräte dort liegen — Fenster,
Tasten, Maus, Hebel, Ablage, Systemauskunft. `rustgame-core` soll **ohne
jede Betriebssystem-Abhängigkeit** bauen und kopflos testen; das ist der
Grund für den Schnitt zwischen den Kisten und bleibt es.

#### Drei Messungen, die in die Irre führen

* **`get_size()` sagt vor dem Start etwas anderes als danach.** Vorher
  nennt pygame den **Wunsch**, nachher die **Wirklichkeit**:
  `Camera(name, (1,1))` gibt vor `start()` `(1, 1)` und danach
  `(320, 240)`. Ich hatte zweimal gemessen, einmal mit und einmal ohne
  `start()`, und zwei richtige Messungen sahen zusammen widersprüchlich
  aus. **Zwei Messungen, die sich widersprechen, messen meist zwei
  Zustände.**
* **Die Masken stimmten, die Tiefe nicht.** Ein Bild über
  `image.frombytes(.., "RGB")` ist eine 24-Bit-Fläche mit den Masken
  `(ff0000, ff00, ff, 0)` — und pygames Kamera gibt eine mit **32** und
  denselben Masken. Gefunden hat es die Probe, nicht der Blick.
* **`format` wird nicht geprüft.** `"YUV"`, `"RGBA"`, sogar `"quatsch"`
  kommen bei `Camera(...)` durch. Umgekehrt nimmt `colorspace` **nur**
  `"YUV"` und `"HSV"` — nicht einmal `"RGB"`. Hier stand das Gegenteil,
  ungemessen, bis die Probe es umwarf.

### Eingebettete Bitmapschnitte

Eine Schrift mit Umrissen darf daneben **fertige Pixelbilder** für
bestimmte Größen mitbringen; die Tabellen heißen `EBLC`/`EBDT`
(schwarzweiß), `CBLC`/`CBDT` (farbig) oder `bloc`/`bdat` (Apple). Der Sinn:
bei kleinen Größen sieht ein von Hand gesetztes Bild besser aus als jede
Rasterung.

Hier stand lange nur die **Größenliste**: `get_sizes()` las aus `EBLC`, wie
viele Pixelgrößen eine Schrift mitbringt, und gerastert wurde trotzdem
immer der Umriss. Jetzt werden die Bilder gelesen, und `PyGameMono.otf` bei
Größe 8 ist Pixel für Pixel das, was FontForge exportiert hat:

```text
   mit Schnitt        ohne Schnitt (Umriss)
     .####.              .#####.
     #....#              #....#.
     #....#              ##...#.
     ######              ######.
     #....#              ##..##.
     #....#              #....#.
     #....#              .......
```

Vier Dinge daran sind nicht offensichtlich:

* **Die Größe muss genau stimmen.** `ttf-parser` sucht von sich aus den
  nächstliegenden Schnitt heraus; FreeType nimmt ein eingebautes Bild nur,
  wenn die Pixelgröße auf den Punkt passt, und rastert sonst den Umriss.
  Eine Zeile, ohne die `PyGameMono.otf` bei Größe 24 das 19er-Bild
  bekäme — vergrößert und matschig.
* **`EBDT`-Format 1 und 2 unterscheiden sich um eine Bytegrenze.** Bei 1
  fängt jede Zeile auf einer solchen an, bei 2 laufen die Bits durch. Bei
  einer Breite, die kein Vielfaches von 8 ist, verschiebt das jede Zeile
  außer der ersten — der Test `zeilenweise_und_gepackt_sind_nicht_dasselbe`
  zeigt es an denselben Bytes mit zwei Bedeutungen.
* **Graustufen werden gestreckt, nicht verschoben.** Aus 2 Bit wird
  0/85/170/255 und nicht 0/64/128/192; sonst wäre kein Pixel je ganz
  deckend.
* **Nicht jeder Stil verträgt sich damit**, und die Trennlinie ist pygames:

  | verträglich | unverträglich |
  |---|---|
  | `vertical`, `wide`, `underline` | `rotation`, `strong`, `oblique` |

  Ein fertiges Bild lässt sich verschmieren und unterstreichen, aber nicht
  scheren und nicht drehen. Dass `wide` auf der verträglichen Seite steht
  und `strong` nicht, ist von `rustgame-font` aus **nicht zu sehen** —
  beide landen dort auf demselben Schalter und unterscheiden sich nur im
  Zuschlag. Also entscheidet die Hülle, und
  `test_freetype_Font_use_bitmap_strikes` prüft jedes Lager einzeln.

Und noch eine Falle, die der Zwischenspeicher aufgestellt hat: die Bilder
liegen unter einem Schlüssel aus Glyphennummer und Stil, und der Schnitt
musste **in den Schlüssel**. Ohne das gab ein `use_bitmap_strikes = False`
nach einem Durchgang mit Schnitten dasselbe Bild zurück — aus dem
Speicher. Genau diesen Vergleich stellt der Test an.

### Zwei Textkisten, zwei Größen

`pygame.font` (SDL_ttf) und `pygame.freetype` liefern bei **derselben
Größe** verschieden große Buchstaben, und das ist kein Fehler:

```text
pygame.font.Font(None, 24).size("Music")           (45, 18)
pygame.freetype.Font(None).get_rect("Music", 24)   (1, 17, 67, 17)
```

Der Grund steht in pygames `font.c`: für die **mitgelieferte** Schrift
rechnet `pygame.font` die Größe mit `0.6875` klein. `Font(None, 16)` ist
also freesansbold in Größe 11 — `get_height()` liefert 11, nicht 16. Wer
den Pfad selbst angibt, bekommt die volle Größe.

`pygame.freetype` tut das **nicht**. Und der Beweis dafür steht in pygames
eigenem `ftfont.py`, das die beiden Kisten versöhnen soll:

```python
resolution = int(self.__get_default_resolution() * 0.6875)   # 49 statt 72
```

Diese Zeile wäre sinnlos, wenn `pygame.freetype` schon klein rechnete.

Hier rechnete es klein, weil beide Wege dieselbe Ladefunktion riefen. Was
dabei herauskam:

| | pygame | vorher hier |
|---|---|---|
| `freetype.get_rect("A", size=24)` | `(0, 17, 17, 17)` | `(0, 12, 12, 12)` |
| `freetype.get_sized_ascender(24)` | `20` | `13` |
| `ftfont.Font(None, 24).size("Music")` | `(47, 19)` | `(31, 13)` |

Die letzte Zeile ist die schlimmste: dort wirkte der Faktor **doppelt**,
einmal beim Laden und einmal über die Auflösung 49. `0.6875² = 0.47`, und
`67 × 0.47 = 31`.

Aufgefallen ist es an `font_test.test_issue_font_alphablit` — einem Test,
der nur prüft, ob die obere linke Ecke eines Textbildes durchsichtig ist.
Der Vermerk nannte ihn eine Rasterkante. Er war keine: das Bild war ein
Drittel zu klein.

### Zwei Stufen, nicht eine

`render_to` legt Text auf eine vorhandene Fläche. Mit einer
Hintergrundfarbe sind das **zwei** Vorgänge, und pygames eigener Test
schreibt sie sogar aus:

```python
# Blitting is done in two stages. First the target is alpha filled
# with the background color, if any. Second, the foreground
# color is alpha blitted to the background.
```

Hier war es einer: `render` backte Grund und Schrift in ein Bild, und das
Bild wanderte als Ganzes auf das Ziel. Bei einem **halbdurchsichtigen**
Grund ist das nicht dasselbe — sein Alpha wirkt dann zweimal, einmal gegen
die Schrift und einmal gegen das Ziel:

```text
Grund (255,120,13,180), Schrift (128,97,213,60), Ziel schwarz
   zwei Stufen        (168, 87, 57)
   ein Bild (vorher)  (174, 88, 46)
```

Dazu kamen an derselben Stelle zwei weitere Funde, und alle drei verdankt
dieses Projekt einem Test, der **nicht gegen pygame vergleicht, sondern
gegen sich selbst**: `render_to_mono` legt die Farben erst mit
`surf.blit` übereinander, merkt sich das Ergebnis und verlangt danach
dasselbe von `render_to`. Beide Seiten sind rustgames Rechnung — sie
müssen zusammenfallen.

* **Das Alpha der Schriftfarbe verschwand**, sobald nicht geglättet wurde.
  Einfarbig gibt `render` eine Fläche mit Zweifarbentafel, und die Tinte
  steht dort mit Alpha 255 — auch bei halbdurchsichtiger Schrift. pygame
  wendet das Alpha erst beim Auftragen an; hier wurde die 255 aus dem Bild
  genommen und die Schrift damit **deckend** aufgetragen.
* **Eine Fläche ohne Alphakanal galt als durchsichtig.** Bei XRGB8888 sind
  die obersten acht Bit null, und die Mischung hielt das Ziel darum für
  leer und **ersetzte** statt zu mischen: `(99, 0, 100)` statt
  `(20, 0, 20)`.
* **Und die Formel war eine eigene.** Keine Rundung traf alle Messwerte:

  | Quelle | pygames `ALPHA_BLEND` | Lehrbuch | gerundet |
  |---|---:|---:|---:|
  | `(97, a=60)` | **23** | 22 | 23 |
  | `(120, a=180)` | **84** | 84 | 85 |
  | `(99, a=64)` | **25** | 24 | 25 |

  `ALPHA_BLEND_COMP` trifft alle drei, weil es auf der 256er-Skala rechnet
  — und es liegt längst in `rustgame_core::blend_formula`, exhaustiv über
  alle 16.777.216 Kombinationen geprüft. **Eine Formel zweimal zu haben
  ist zweimal die Gelegenheit, sich zu unterscheiden.**

Und der Kasten eines Leerzeichens, der dieselbe Prüfung eröffnet hatte:
`get_rect(" .")` gab `(9, 3, 3, 3)`, pygame `(0, 3, 11, 3)`. Ein leeres
Zeichen am Rand wird nicht weggeschnitten — und wie weit es zählt, hängt
von der Glättung ab:

```text
test_sans, Größe 24      geglättet      einfarbig
  "."                    (2, 3,  3, 3)  (2, 3,  2, 3)
  " ."                   (0, 3, 12, 3)  (0, 3, 11, 3)
  ". "                   (2, 3, 12, 3)  (2, 3,  6, 3)
  " . "                  (0, 3, 21, 3)  (0, 3, 15, 3)
```

Geglättet belegt es `[Stift, Stift + Vorschub]`, einfarbig nur
`[Stift, Stift + 1]`. Vier Fälle, zwei Zweige, jeder gemessen.

#### Und zuletzt: zweimal **schreiben**, nicht zweimal rechnen

Mit Hintergrundfarbe geht der Grund erst **in die Fläche**, und die
Schrift mischt sich dann gegen das, was dort wirklich steht. Auf einer
Fläche mit wenigen Bit je Kanal ist das ein Unterschied — der Grund rastet
beim Ablegen auf eine Farbstufe ein, und die Schrift rechnet ab dieser
Stufe weiter:

```text
RGB565, Grund (255,120,13,180), Schrift (128,97,213,60)
  zweimal schreiben (pygame)   Grün 89
  zweimal rechnen              Grün 85
```

Eine Farbstufe, und sie fällt nur in 565 auf — genau dort, wo pygames
eigener Test hinsieht.

**Drei Proben halten das jetzt fest** (`vergleich_module.py`):
`freetype.Groesse` (zehn Zeichen und acht Größen auf den Wert),
`freetype.Leerkasten` (vierzehn Zeichenketten in beiden Zweigen) und
`freetype.render_to Stufen` (fünfzehn Kombinationen aus Tiefe, Flaggen und
Grund) — dazu `font.Standardschrift`, die den **Arbeitsordner wechselt**
und danach alle vier Wege zur mitgelieferten Schrift prüft. Alle vier
stimmen Wert für Wert. **Ein Fix ohne Probe ist ein Fix,
der still verschwinden kann** — davon handelt der halbe Bericht hier.

### Das Puffer-Protokoll — und wo `unsafe` wirklich steht

Der Plan hat „kein C-API, kein Puffer-Protokoll" von Anfang an als Grenze
genannt, und sie kostete **zweiundfünfzig Tests** in acht Dateien — mehr
als alle anderen offenen Punkte zusammen. Sie ist jetzt weg, und der Weg
dorthin ist der lehrreichste Teil dieses Zuges.

**Ein Puffer-Protokoll ohne rohen Zeiger gibt es nicht.** CPython fragt
über den Schlitz `bf_getbuffer` nach einer Adresse, einer Länge und zwei
Zahlenfeldern und behält sie, bis es den Puffer freigibt. Wer statt dessen
kopiert, beantwortet eine andere Frage: `memoryview(s.get_buffer())[0] = 1`
soll die Fläche ändern, nicht ein Abbild davon.

#### „`unsafe` in genau einer Datei" — und warum der Satz falsch war

So stand er hier, und er klang gut. Nachgezählt — und zwar nach echten
`unsafe {`- und `unsafe fn`-Stellen, nicht nach dem Vorkommen des Wortes —
liegt `unsafe` in **sieben** Dateien:

| Datei | Stellen | was dort steht |
|---|---:|---|
| `rustgame-py/src/puffer.rs` | 21 | **die Arbeit**: die Seite, die rustgame ausgibt |
| `rustgame-py/src/neuerpuffer.rs` | 11 | **die Arbeit**: die Seite, die ein Python-Programm ausgibt (`pygame.newbuffer`) |
| `farbe.rs`, `maske.rs`, `pixelfeld.rs`, `pufferzeiger.rs`, `ton.rs` | je 4 | **nichts** — die beiden Schlitze `__getbuffer__`/`__releasebuffer__` samt ihren zwei Aufrufen nach `puffer.rs` |

PyO3 schreibt diese Schlitze als `unsafe fn` vor; daran ist nichts zu
machen, und ihr ganzer Körper ist eine Weiterleitung. Richtig ist also:
**`unsafe` arbeitet in zwei Dateien, und fünf reichen es durch.**

Der Satz hat übrigens ein zweites Mal veraltet: er nannte lange *eine*
Datei, dann — nach dem Nachzählen — `puffer.rs` allein. Mit
`neuerpuffer.rs` sind es zwei. **Eine Zahl im Text ist eine Zusage, die
jemand nachhalten muss**; hier hält sie der Befehl, der sie erzeugt:

```bash
grep -rEc "unsafe\s*(\{|fn |impl )" rustgame/crates/*/src/*.rs | grep -v ":0$"
```

(`grep -c` zählt **Zeilen** mit Treffer, nicht Treffer — in `puffer.rs`
stehen auf einer Zeile zwei. Die Tabelle nennt die Zahl, die dieser Befehl
ausgibt, damit beide dasselbe sagen.)

Der zweite Halbsatz war noch schlechter: „die sechs anderen Kisten tragen
weiter `#![forbid(unsafe_code)]`". **Dieser Lint stand nirgends im
Quelltext.** Die Aussage war inhaltlich wahr — in den sechs Kisten gibt es
kein `unsafe` —, aber nichts hielt sie fest. Eine Behauptung, die niemand
nachhält, veraltet still; genau davon handelt der Abschnitt „Der
wiederkehrende Fehler" weiter unten.

Jetzt hält sie der Übersetzer:

```text
rustgame-core, -display, -font, -math, -mixer, -sprite
    #![forbid(unsafe_code)]
    #![deny(missing_docs)]
rustgame-py
    #![deny(unsafe_code)]        scharf, mit fünf einzelnen Ausnahmen
    #![deny(missing_docs)]
rustgame-py/src/puffer.rs
    #![allow(unsafe_code)]       die eine erlaubte Stelle
```

Wer eine siebte Stelle anlegt, muss sie ausdrücklich erlauben — und das
fällt beim Lesen auf.

#### Und derselbe Befund beim zweiten Lint

`#![deny(missing_docs)]` stand in **zwei** von sieben Kisten, und auch das
war kein Vorsatz, sondern ein Versehen: `rustgame-display` und
`-mixer` hatten ihn, die anderen fünf nicht. Eingeschaltet fehlten **138**
Doku-Zeilen.

Der Verdacht war Fleißarbeit; die Messung sagte etwas anderes. Über die
Hälfte kam aus **Makros** — eine Zeile in `blend.rs` deckte siebzehn
Mischmodi ab, eine in `gfxzeichnen.rs` siebzehn Grundformen. Und `frect.rs`
war der Zwilling eines dokumentierten Typs: `Rect` hatte seine Doku,
`FRect` nicht, obwohl beide aus demselben Makro kommen. Dort steht jetzt,
was die Wahrheit ist — `Siehe [Rect::move_]` plus das, was anders ist
(`f32` statt `i32`). **Zwei Beschreibungen derselben Sache sind zwei
Gelegenheiten, auseinanderzulaufen.**

Was blieb, waren Felder und Konstanten, die wirklich eine Zeile verdienen:
`Vector2::x` („der Anteil nach rechts"), `Color::a` („Deckung — 255 ist
voll deckend"), die zwölf `PyBUF_*`-Zahlen aus CPythons `object.h`.

Drei Zusagen müssen dort eingehalten werden, und alle drei stehen im
Modulkopf: die Adresse überlebt den Puffer (eine Fläche wechselt nach dem
Anlegen nie ihre Größe), der Halter bleibt am Leben (`view.obj` mit
erhöhtem Zähler), und `shape`/`strides` gehören dem Ausgeber (auf dem
Haufen, in `freigeben` wieder eingesammelt).

**Was gemessen ist**, und die zweite Zeile ist die, auf die man nicht kommt:

```text
s = Surface((10, 6), 0, 32); a = s.get_view("2")
Importer(a, PyBUF_SIMPLE).ndim      0      shape/strides None
Importer(a, PyBUF_ND)               BufferError
Importer(a, PyBUF_STRIDES).strides  (4, 40)
Importer(a, PyBUF_RECORDS).format   "=I"
Importer(a, PyBUF_C_CONTIGUOUS)     BufferError
Importer(a, PyBUF_F_CONTIGUOUS)     geht - shape (10,6), strides (4,40)
```

`PyBUF_ND` allein **scheitert**, obwohl `PyBUF_SIMPLE` durchgeht: `SIMPLE`
fragt nach einem Stück Speicher, und das ist die Fläche; `ND` fragt nach
einem Feld ohne Schrittweiten, und dafür müssten die Zeilen lückenlos
aneinanderliegen — im Sinne von `(Breite, Höhe)` tun sie das nicht. Die
zweidimensionale Sicht ist **spaltenweise** zusammenhängend, nicht
zeilenweise. Wer sie als C-Feld anmeldet, bekommt von numpy ein gedrehtes
Bild.

Vier weitere Befunde, die keine Messreihe hergibt:

* **`get_buffer()` sperrt sofort, `get_view(...)` erst beim ersten
  Zugriff.** Gemessen: nach `get_view("2")` ist `get_locked()` falsch, nach
  `v.length` wahr. Und die Sperre ist der `BufferProxy` **selbst** —
  `get_locks()` gibt ihn heraus.
* **`BufferProxy(objekt)` fragt beim Anlegen nichts.**
  `base_test.test_PgObject_GetBuffer_array_interface` baut darauf:
  `BufferProxy(o)` und dann `assertRaises(NoDictError, o.is_dict_alive)` —
  erst `v.length` liest die Feld-Schnittstelle. Und `BufferProxy(1)` geht
  durch; erst `bp.length` gibt den `ValueError`.
* **Die Reihenfolge ist Puffer, dann `__array_struct__`, dann
  `__array_interface__`** — und jeder Weg wird bei einem Fehlschlag
  **stillschweigend** aufgegeben. Zwei Tests prüfen es von beiden Seiten:
  einer gibt `__array_interface__ = property(lambda self: None)` mit, der
  andere `__array_struct__ = property(lambda self: None)`.
* **Ein Format geht unverändert durch.** `"1h"` ist nach PEP 3118 dasselbe
  wie `"h"`, aber ein Verbraucher bekommt es **wörtlich** zurück; `">h"`
  trägt eine gedrehte Byteordnung; `"hh"` ist ungültig und muss ein
  `ValueError` werden. Eine Tabelle statischer Formate kann keines der
  drei — hier stand eine, und sie war an allen drei Stellen falsch.

### Der Pixelspeicher: die Grenze, die trotzdem gefallen ist

Hier stand ein Abschnitt mit der Überschrift **„Die eine Grenze, die
bleibt"**. Er rechnete vor, was es kosten würde, `PixelBuf` von
`Vec<u32>` auf einen Byte-Vorrat umzubauen — 55 Zugriffsstellen, die
innerste Blitschleife, 123 Referenzbilder, 17 Mischmodi — und schloss:
0,8 % der Tests seien das nicht wert.

**Die Rechnung stimmte, der Schluss nicht.** Sie war eine Aufwandsschätzung
und keine Messung, und sie hat die falsche Frage beantwortet: nicht „wie
viele Tests", sondern „wie viel Wahrheit". Eine Engine, die `get_bitsize()`
mit 8 beantwortet und im Speicher vier Bytes hält, ist an der Stelle kein
Ersatz, sondern eine Ähnlichkeit.

#### Was jetzt dort liegt

`bpp` Bytes je Pixel, `pitch` Bytes je Zeile (auf vier aufgerundet),
niedrigstes Byte zuerst — SDLs Anordnung. Der Wert eines Pixels ist der
**abgebildete**: bei einer Palettenfläche der Index, bei 16 Bit das
RGB565-Wort, bei 32 Bit das ARGB-Wort. Gerechnet wird weiter in ARGB;
`PixelBuf::nach_argb` und `von_argb` sind die Übergänge, und bei ARGB8888
sind beide die Kennabbildung — der 32-Bit-Weg ist Byte für Byte derselbe
wie vorher.

Der Vorrat bleibt ein `Vec<u32>` und wird byteweise benutzt. Der Grund ist
nicht Bequemlichkeit: ein `Vec<u8>` darf an einer ungeraden Adresse
anfangen, `Surface._pixels_address` wandert nach Python und von dort in
numpy, und die Feld-Schnittstelle behauptet `PAI_ALIGNED`.

#### Was es gebracht hat

| | vorher | jetzt |
|---|---:|---:|
| `surface` | 132 / 141 | **137 / 137** |
| `pixelarray` | 39 / 44 | **44 / 44** |
| `pixelcopy` | 8 / 12 | **12 / 12** |
| `surfarray` | 13 / 20 | **19 / 20**\* |
| benannte Grenzen | 11 | **14**, davon 12 als Probe |

\* die zwanzigste braucht ein Fenster und überspringt sich selbst.

Und eine Grenze, die niemand auf der Liste hatte, fiel mit: der
**Tafelindex**. Er stand dort als eigener Eintrag — „auf einer
Palettenfläche überlebt der Index den Weg durch den Speicher nicht", weil
pygames Standardtafel 256 Einträge und nur 128 verschiedene Farben hat.
Seit der Index selbst im Speicher liegt, überlebt er.

#### Was es gekostet hat

Die Schätzung lag bei den Zahlen richtig und beim Risiko daneben. 34
Zugriffsstellen in `surface.rs`, acht Aufrufer von `mit_pixeln` in drei
weiteren Dateien — und **kein einziger** der 123 Referenzvergleiche, der
251 `draw`-Tests oder der 17 Mischmodi ist dabei rot geworden. Der Grund
ist der, der die Rechnung hätte sparen können: bei 32 Bit ist die
Umrechnung die Kennabbildung, und 32 Bit ist das, worin fast alles
gemessen wird.

Was wirklich Arbeit machte, waren die **Folgestellen** — Stellen, die
einen Pixelwert durch eine Farbe schickten und zurück:

* `Surface::gepackt_an` rechnete Index → Farbe → nächster Index. Auf einer
  Tafel mit doppelten Farben ist das nicht die Umkehrung.
* `Surface::wie` setzte `s.format` am Puffer vorbei — die neue Fläche
  meldete acht Bit und lag in vier Bytes.
* `PixelArray::roh` las über `get_at_mapped` (ARGB) und schrieb über
  `set_at_ungeprueft` (auch ARGB), während der Vergleich schon gepackt
  war.

Alle drei sahen im Quelltext richtig aus und waren es einen Tag vorher
auch. **Eine Datenstruktur zu ändern heißt, jede Stelle zu prüfen, die
ihre alte Bedeutung kannte** — und das sind nicht die Stellen, die sie
anfassen, sondern die, die sie voraussetzen.

### Der Klangspeicher: dieselbe Entscheidung, dieselbe Auflösung

`Klang` hielt seine Abtastwerte **immer** 16-bittig, ganz gleich, womit der
Mixer lief. Nach außen stimmte das meiste: `get_raw()` rechnete beim
Herausgeben um, `get_length()` zählte richtig. Nur die **Adresse** log —
`Sound._samples_address` zeigte auf 16-Bit-Worte, während
`__array_interface__` unter `init(size=8)` `|u1` versprach.

Solange niemand dem Zeiger folgte, fiel es nicht auf. Seit es
`pygame.sndarray` gibt, folgt ihm jemand: `sndarray.samples` ist eine
**lebende** Sicht, und was hineingeschrieben wird, steht danach im Klang.
Mit zwei Ablagen ist das nicht zu machen.

Jetzt liegt dort, was SDL_mixer in `chunk->abuf` hält. Gemischt wird weiter
16-bittig; die Umrechnung geschieht **einmal je `play()`** und nicht je
Puffer — der Treiber kopiert die Werte ohnehin in seinen eigenen Puffer,
es kommt also keine Kopie dazu.

Der Fund dabei, den man sich nicht ausdenkt: `Sound(array=...)` schiebt
schmale Werte nach links, damit sie den Wertebereich ausfüllen, und
schneidet breite **ohne** Verschiebung ab. pygames eigener Test nennt das
ausdrücklich unsymmetrisch:

```python
lshift = abs(format) - 8 * a.itemsize
if lshift >= 0:
    # This is asymmetric with respect to downcasting.
    a3 <<= lshift
```

Und `format` ist dabei das des **Mixers**, nicht 16. Hier stand `16 -
bits`, weil der Klang damals immer 16-bittig lag; unter `init(size=8)`
wurde ein `uint8`-Feld damit erst hochgeschoben und beim Ablegen wieder
heruntergerechnet — mit dem Nullpunktversatz von 128 dazwischen. Aus
`[0, 15, 240, 255]` wurde `[128, 143, 112, 127]`.

### Ein Blit, der aussah wie ein Kopiervorgang

`BLEND_ALPHA_SDL2` hatte im Verteiler **keinen Zweig**, und der `_`-Fall
gab es stillschweigend an pygames Alphamischer weiter. Bei Quell-Alpha 0
kam damit die Quelle heraus, wo das Ziel hätte stehenbleiben müssen — um
bis zu 255 daneben, und ohne jeden Hinweis.

Die Formel dazu ist der lehrreichste Fund des Projekts. SDL rechnet dort
**Rot und Blau gemeinsam in einem 32-Bit-Wort**:

```c
s1 = s & 0xff00ff;                       /* Rot und Blau zusammen */
d1 = d & 0xff00ff;
d1 = (d1 + ((s1 - d1) * alpha >> 8)) & 0xff00ff;
```

Ist das blaue Byte der Quelle kleiner als das des Ziels, **borgt** die
Subtraktion beim roten, und die Maske räumt es hinterher weg. Deshalb kann
**keine** kanalweise Formel es treffen. Ich habe es trotzdem versucht:
neun Kandidaten mit acht Divisionskniffen, 72 Paare — der beste lag bei
480 von 1728 falsch. Der erste Versuch, aus pygames Erwartungstabelle
*erschlossen*, stimmte auf vier nachgerechneten Fällen und lag gegen die
ganze Tabelle bei 40 von 64 daneben. Vier Treffer aus Zufall. Die
Übertragung traf auf Anhieb: **0 von 108.000** bei den Farbkanälen.

Und dann noch die Umkehrung derselben Lehre: die Werte in pygames
`results_expected` weichen auf dieser Maschine um bis zu **2** ab, weil
die Tabelle von einer anderen SDL-Fassung stammt — der Test prüft deshalb
mit `delta=4`. Ich hatte sie als Sollwert genommen und beinahe eine
**richtige** Umsetzung „berichtigt", bis die Gegenmessung an echtem pygame
zeigte, dass genau das herauskommt, was hier schon stand. **Fremde
Erwartungswerte sind Messwerte einer anderen Maschine, keine Vorschrift.**

### Die Kiste, die kein Werkzeug je gesehen hat

Vier Prüfungen laufen über den Workspace: `cargo test --workspace`,
`cargo clippy --workspace`, `cargo doc --workspace`, und dazu die
Python-Werkzeuge. **`rustgame-py` ist in keiner davon.**

Die Kiste hat eine eigene `[workspace]`-Tabelle und steht damit außerhalb —
mit gutem Grund: `extension-module` bindet absichtlich kein libpython ein,
und wäre sie Mitglied, würde `cargo test --workspace` für den ganzen Rest
scheitern. Der Grund stimmte. Die Folge hat niemand nachgehalten:

```text
cargo test   --release   in crates/rustgame-py   →  3 Typfehler im Testcode
cargo clippy --release   in crates/rustgame-py   →  18 Hinweise
cargo doc    --no-deps   in crates/rustgame-py   →  15 Hinweise
```

Die drei Typfehler standen in `puffer.rs` in einem `mod tests`, der seit
seiner Entstehung **nicht einmal übersetzt** wurde: `shape` und `strides`
wurden als `Vec<i64>` gebaut, wo `Vec<isize>` hingehört, und `format`
bekam ein `&CStr` statt eines `CString`. Ein Test, der nicht übersetzt,
ist schlechter als keiner — er sieht in der Datei aus wie eine Zusage.

Zwei der 18 Clippy-Hinweise waren inhaltlich: `unsafe fn fuellen` und
`unsafe fn freigeben` in `puffer.rs` hatten **keinen `# Safety`-Abschnitt**.
Die Begründung stand daneben im Fließtext — gut lesbar für Menschen,
unsichtbar für das Werkzeug, und genau an der Stelle, an der diese Datei
behauptet, jedes `unsafe` trage einen Satz mit seinem Grund.

Seitdem gehören beide Aufrufe in der Kiste zur Prüfliste, und
`tools/einbauen.py` ruft den Bau dort, wo er hingehört.

### Die Falle mit der gebauten Datei — drei Mal derselbe Fehler

**Gemessen wurde dreimal ein Modul, das nicht das gebaute war.** Jedes Mal
anders verpackt, jedes Mal dieselbe Sorte:

| | wie es aussah | was wirklich war |
|---|---|---|
| 1 | „2 Fehler" in drei Läufen hintereinander | `cargo build \| grep -c '^error'` gibt bei **null** Treffern den Rückgabewert 1, also lief das `cp` im `&&` nicht |
| 2 | ein Fehlertext, den es im Quelltext nicht mehr gab | **zwei Kopien** der `.pyd`; `vergleich_module.py` legt `crates/rustgame-py` **vor** `site-packages` in den Suchpfad |
| 3 | ein Bau, der „durchlief" und nichts tat | `cargo build -p rustgame-py` vom Workspace aus: `error: package ID specification did not match any packages` — die Kiste steht außerhalb |

Der zweite Fall fiel nur auf, weil der Fehlertext zu einem Entwurf gehörte,
den ich eine halbe Stunde vorher gelöscht hatte. Ohne diesen Zufall hätte
ich die Schnittstelle nach einer Messung gebaut, die ein Modul von vorgestern
beschrieb.

Daraus ist `tools/einbauen.py` entstanden, und es setzt eine Regel durch:
**wer baut, legt die Datei an jede Stelle, von der sie geladen werden kann,
und prüft danach per SHA-256, dass sie wirklich dort liegt.** Nicht der Bau
wird geprüft, sondern das Ergebnis — unter Windows lässt sich eine `.pyd`,
die ein laufendes Python geladen hat, nämlich gar nicht ersetzen, und das
Kopieren schlägt dann still fehl.

```bash
.venv/Scripts/python.exe rustgame/tools/einbauen.py            # bauen + legen
.venv/Scripts/python.exe rustgame/tools/einbauen.py --pruefen   # nur nachsehen
```

### Der zehnte blinde Fleck: Konstanten in Untermodulen

`api_coverage.py` sagte **1263 von 1263 (100 %)**. Gleichzeitig fehlten:

```text
pygame.font.UCS4               1
pygame.base.HAVE_NEWBUF        1
pygame._freetype.BBOX_EXACT    0   (und drei Geschwister)
pygame.midi.MIDIIN             32771
pygame.sysfont.is_init         False
```

Zwei Filter hatten sie weggenommen, und **beide waren Listen von Hand**:

* Wer kein Aufruf und kein Modul war, fiel als „Konstante" heraus — mit dem
  Verweis, die kämen aus `gen_konstanten.py`. Das Werkzeug erzeugt aber nur
  die Konstanten des Hauptmoduls und von `locals`, nicht die der
  Untermodule. **Niemand fragte sie.**
* Eine Menge `EGAL` nahm sieben Namen aus (`os`, `sys`, `platform`,
  `warnings`, `ABC`, `abstractmethod`, `errorfnc`), weil „kein Programm
  `pygame.camera.os` schreibt".

Dieselbe Handliste stand in `vergleich_module.py` ein zweites Mal — acht
Namen, dieselbe Begründung. Zwei Messgeräte, ein blinder Fleck.

Beide Listen sind weg. Die neue Probe **`Untermodule gesamt`** stellt
stumpf `dir()` gegen `dir()` für alle **60** Untermodule, ohne Ausnahme,
und `api_coverage.py` zählt jeden öffentlichen Namen. Ergebnis:

| | vorher | jetzt |
|---|---:|---:|
| API-Namen | 1263 | **3245** |
| Untermodule, die auseinandergingen | — | **18 gefunden, 18 geschlossen** |

Und nach dem elften und zwölften Fleck (siehe unten) noch einmal von 3245
auf **3471**, mit 21 Klassen und drei leeren Klassen als Fund.

Die Zahl ist um 1982 gewachsen und steht weiter auf 100 %. Was dabei
herauskam, war **nicht** nur Kleinkram:

**`pygame._freetype` ist ein eigenes Modul.** Hier stand
`m.add("_freetype", m.getattr("freetype")?)` — ein Name, zwei Zeiger auf
dasselbe Modul. Naheliegend, denn `freetype.py` ist bei pygame wirklich nur
eine Hülle. Gemessen sind es zwei Namensräume mit sieben Unterschieden:
`freetype` hat `SysFont`, `get_fonts`, `match_font`, `_freetype` hat die
vier `BBOX_*`. `freetype.py` holt seine Namen also **einzeln** und nicht
mit `import *`.

**`pygame.math.VectorIterator` gab es nicht.** `iter(Vector2())` lieferte
einen `tuple_iterator`: `__iter__` baute ein Tupel aus den Zahlen und gab
dessen Iterator heraus. Das läuft — und ist in zwei Dingen falsch. Der Typ
ist der eine; der andere ist gemessen:

```python
w = pygame.Vector2(1, 1)
i = iter(w)
w.x = 99
list(i)        # [99.0, 1.0]  -  nicht [1.0, 1.0]
```

**Der Iterator hält den Vektor, nicht seine Zahlen.** Ein Tupel hätte die 1
festgehalten.

**PyO3s `PyModule::add` schreibt den Namen auch nach `__all__`.**
`dir(rustgame_py._sdl2)` hatte darum **59** Einträge mit acht Dubletten, wo
pygame 51 hat: `sdl2_init.py` führt die acht Untermodule in `__all__`, und
die acht `add`-Aufrufe trugen sie ein zweites Mal ein. `dir()` entdoppelt
nicht, wenn ein Modul sein `__dir__` selbst stellt. Seitdem `setattr`.

**Ein zweites `__getattr__` ersetzt das erste still.** Beim Nachtragen der
fehlenden Namen habe ich in `ftfont.py` und `midi.py` je ein zweites
`__getattr__` *oben* in die Datei gesetzt — und Python nimmt die **letzte**
Zuweisung. `dir(midi)` zeigte `pygame`, `getattr(midi, "pygame")` warf einen
`AttributeError`. Gefunden hat das `pygame_tests.py`, das beim Umbiegen der
Modulnamen jeden Namen einmal anfasst. Seitdem prüft ein Sieben-Zeiler, dass
in keiner eingebetteten Datei ein Dunder zweimal steht.

In `midi.py` kam ein zweiter Fund dazu: `MIDIIN` und `MIDIOUT` waren
**erreichbar** (ein `__getattr__` am Dateiende holte sie vom Wurzelmodul) und
nur in `dir()` nicht zu sehen. Mein erster Flicken schrieb die Zahlen 32771
und 32772 als Literale daneben — zwei Quellen für dieselbe Zahl, also eine
zuviel. Sie sind wieder weg.

### Ein Messgerät, dessen Ausgabe man kürzt, darf lügen

`vergleich_module.py` schnitt jede Abweichung bei 200 Zeichen ab. Bei einer
Probe, die vierzig Einträge zurückgibt, heißt das: man sieht, **dass** etwas
anders ist, und nicht **was**. Drei Abweichungen standen so eine Weile da,
ohne verwertbar zu sein.

Jetzt gibt es `--genau <Teilname>`, und dahinter eine Funktion, die zwei
Werte rekursiv gegeneinanderstellt:

* bei Listen: Stelle für Stelle, mit dem Weg dorthin (`[3][1]`),
* bei einem Paar `(Name, Wert)` mit gleichem Namen: **der Name wird der
  Weg** (`{_freetype}`) — ohne das zählt man die Liste mit dem Finger ab,
* bei zwei sortierten Namenslisten: **als Menge** (`fehlt … zuviel …`), denn
  ein einziger fehlender Name verschiebt sonst die ganze Liste und die
  Ausgabe ist zwanzig Zeilen Rauschen,
* bei langem Text: die **erste** abweichende Stelle, mit dreißig Zeichen
  Umgebung.

Die erste Fassung gab zwei Zeilen je Eintrag aus (`fehlt` und `zuviel`
untereinander) — bei achtzehn Einträgen waren sie einander nicht mehr
zuzuordnen. Jetzt eine Zeile.

Gefunden hat dieses Werkzeug sich selbst bezahlt: von den 18 Modulfunden
wären ohne es höchstens drei sichtbar geworden.

### `pygame.newbuffer` — die Schnittstelle, die man erraten kann

Die letzten zwei Namen der ganzen API waren `newbuffer.Py_buffer` und
`newbuffer.BufferMixin`: eine Hülle um CPythons `Py_buffer`-Struktur und
eine Basisklasse, mit der eine **Python**-Klasse zum Puffer-Ausgeber wird.
pygame prüft damit sein eigenes Puffer-Verhalten
(`tests/test_utils/buftools.py`).

Der erste Entwurf nahm `view.format = "B"` und `view.shape = [4]` an. Er
lief, er war vollständig, und er war falsch. Gemessen schreibt pygames
eigenes Testwerkzeug das:

```python
view.buf     = self.buf                  # void*       als int
view.format  = addressof(self._format)   # char*       als int
view.shape   = addressof(self._shape)    # Py_ssize_t* als int
view.suboffsets = None                   # NULL
```

**Alle sechs Zeigerfelder sind Zahlen oder `None`.** Wer die Werte hält,
auf die sie zeigen, ist Sache des Aufrufers. Die zweite Fassung legt deshalb
*nichts* an — der erste Entwurf musste seine `CString`s und `Vec`s in
`view.internal` parken, damit sie den Aufruf überdauern.

**Eine Schnittstelle, die man erraten kann, ist noch nicht gemessen.**

Dazu drei gemessene Eigenheiten, die nicht herzuleiten sind:

**Das Fehlerfeld ist 400 Zeichen breit.** pygames Text lautet
`property %400s is undefined for an unallocated view`, und `%400s` richtet
*rechts* aus: zwischen `property` und dem Feldnamen stehen 397 Leerzeichen,
die Meldung ist 446 Zeichen lang und sieht im Terminal aus wie ein
Zeilenumbruch. Nachgebaut, Zeichen für Zeichen.

**`strides` meldet `shape`.** Eine leere Sicht nach `strides` gefragt
antwortet `property shape is undefined …` — ein übersehener Kopiervorgang in
`newbuffer.c`. Nachgebaut, mit demselben Grund.

**pygame nullt CPythons Struktur nicht.** Ein `_get_buffer`, das ein Feld
nicht setzt, lässt dort stehen, was auf dem Stapel lag. Derselbe Fall,
dreimal hintereinander gemessen:

```text
MemoryError
(nbytes 0, b"")
ValueError: memoryview: number of dimensions must not exceed 64
```

Setzt man `buf` und `len`, aber **nicht** `obj`, kommt auch ein Absturz
heraus. Hier wird die Struktur genullt, bevor der Rückruf sie sieht, und ein
unvollständiges `_get_buffer` bekommt eine klare Antwort statt einer
zufälligen. Die beiden Fälle stehen deshalb **nicht** in den Proben: ein
Referenzwert, der vom Stapelinhalt abhängt, misst nichts — und eine Probe,
die den Vergleichslauf abschießt, misst weniger als nichts.

`pygame.newbuffer.Py_buffer(1)` stürzt pygame übrigens ebenfalls ab
(`pygame_parachute`, Speicherzugriffsfehler). Hier gibt es dafür einen
`TypeError`. **Ein Absturz ist das einzige Verhalten, das nicht nachgebaut
wird.**

Damit tut `unsafe` in **zwei** Dateien wirkliche Arbeit — `puffer.rs` für
die Seite, die rustgame ausgibt, und `neuerpuffer.rs` für die Seite, die ein
Python-Programm ausgibt. Fünf weitere Dateien reichen nur Slots durch.

### Der elfte blinde Fleck: die Klassen standen in einer Liste

`alle_untermodule()` liest die Module seit dem neunten blinden Fleck. Über
dieser Funktion stand ein Satz:

> `MODULE` ist von Hand gepflegt, und das ist für **Klassen** richtig:
> `math.Vector3` steht dort, weil jemand gemerkt hat, dass `Vector2` allein
> zu wenig ist.

Der Satz klang vernünftig und war der nächste bequeme Platz. Nachgezählt:

```text
MODULE fragt      24 Klassenobjekte
erreichbar sind   45
nie gefragt       21  -  zusammen 354 Namen
```

Nichts Entlegenes darunter: **`pygame.error` selbst**, `ftfont.Font` mit 63
Namen, `joystick.Joystick`, `key.ScancodeWrapper`,
`math.VectorElementwiseProxy`, `version.SoftwareVersion`,
`system.PowerState`, beide `pypm`-Klassen. Seit `alle_klassen()` wird auch
diese Liste gelesen — dedupliziert **nach dem Objekt**, denn fast jede
Klasse steht unter mehreren Wegen (`pygame.Channel`, `pygame.mixer.Channel`
und `pygame.mixer.ChannelType` sind **eines**; nach Namen gezählt käme 674
heraus, davon 320 Dubletten).

#### Und der zwölfte, im Werkzeug, das keine Stunde alt war

Die erste Fassung von `alle_klassen()` fragte
`__module__.startswith("pygame")`. Gemessen:

```text
pygame._sdl2.controller.Controller.__module__      'builtins'
pygame._sdl2.controller_old.Controller.__module__  'pygame._sdl2.controller_old'
```

**Cython schreibt `builtins`**, wenn die `.pyx`-Datei kein Modul nennt. Die
neue `Controller`-Klasse — fünfzehn Namen, eine mehr als die alte — fiel
damit durch das Netz, und die Tabelle sagte trotzdem 100 %. Die Regel ist
jetzt lesend: **eine Klasse gehört anderswohin nur dann, wenn sie dort auch
zu finden ist.** `abc.ABC` behauptet `abc` und steht dort — fremd.
`Controller` behauptet `builtins` und steht dort nicht — unsere.

### Drei Klassen, die leer waren

Was die gelesene Klassenliste zutage brachte, war kein Kleinkram.

**`joystick.Joystick` hatte null Methoden.** Daneben stand der Vermerk,
dieses Modul könne *vollständig* sein, obwohl es keinen Treiber gibt: es
gebe nie einen Hebel, und was pygame in genau diesem Zustand tut, sei
gemessen. Die **elfte Rasterkante** — gemessen hat `JoystickType`
neunzehn Namen (`get_axis`, `get_button`, `get_hat`, `get_name`, `rumble`,
`set_led`, …), und keiner war da. Kein Test hat es gefunden:
`joystick_test.py` überspringt sich ohne Gerät.

Dazu der zweite Teil desselben Fundes: **`Joystick` ist eine Funktion.**

```text
pygame.joystick.Joystick      <built-in function Joystick>
pygame.joystick.JoystickType  <class 'pygame.joystick.Joystick'>
Joystick is JoystickType      False
```

Hier war beides derselbe Name — mit dem Vermerk, „ein zweiter, leerer Typ
wäre eine Nachahmung ohne Nutzen". Der zweite Typ ist aber nicht leer. Und
die Fabrik hat drei gemessene Fehlertexte, die man nicht herleitet:

| Aufruf | pygame |
|---|---|
| `Joystick(0)`, `Joystick(5)` | `error: Invalid joystick device number` |
| `Joystick(-1)`, `Joystick(-5)` | `error: There are 0 joysticks available` |
| `Joystick()` | `TypeError: function takes exactly 1 argument (0 given)` |
| `Joystick(2**31)` | `OverflowError: Python int too large to convert to C long` |

Dass eine **negative** Nummer einen anderen Satz gibt als eine zu große,
findet man nur durch Hinsehen. `2**31` zeigt außerdem, dass pygame hier ein
C-`long` nimmt — unter Windows 32 Bit.

**Beide `Controller`-Klassen hatten null Methoden**, 15 und 14. Auch hier
kein roter Test, auch hier ein Vermerk daneben. Und sie unterscheiden sich
in jedem Satz:

| | `controller` (neu) | `controller_old` |
|---|---|---|
| vor `init()` | `Controller system not initialized` | `gamecontroller system not initialized` |
| nach `init()` | `Invalid index` | `Index is invalid or not a supported joystick.` |
| `Controller()` | `function missing required argument 'device_index' (pos 1)` | `__init__() takes exactly 1 positional argument (0 given)` |
| `Controller(0.5)` | `TypeError` | **geht** — Cython schneidet ab |
| `Controller("x")` | `'str' object cannot be interpreted as an integer` | `an integer is required` |
| ableitbar | **nein** | ja |

Der vorletzte Punkt kostete einen Umbau: eine Python-Klasse ist immer
ableitbar, also sperrt `__init_subclass__` sie — mit pygames Wortlaut
`type 'Controller' is not an acceptable base type`.

**Und beide Module teilen ein Teilsystem.** Gemessen:

```text
controller.init()      ->  controller_old.get_init()  True
controller.quit()      ->  controller_old.get_init()  False
controller_old.init()  ->  controller.get_init()      True
```

Hier standen zwei **unabhängige** Schalter, und das fiel keiner Probe auf:
beide Module sahen für sich richtig aus. Gefunden hat es eine Gegenprobe,
in der die neue Fassung vor der alten lief — und die alte danach einen
anderen Fehlertext gab als erwartet. *Ein Fehler, der von der Reihenfolge
abhängt, sieht aus wie Zufall.*

**`key.ScancodeWrapper` ist ein `tuple`.** Hier war es eine eigenständige
`#[pyclass]` mit `__len__` und `__getitem__`, und das sah vollständig aus —
`len`, Zugriff und das abgelehnte Durchlaufen stimmten alle. Es fehlten
`count`, `index` und das `isinstance(k, tuple)`:

```text
isinstance(get_pressed(), tuple)   True
get_pressed().count(False)         512
get_pressed() + get_pressed()      gewöhnliches 1024er-Tupel
repr(W((7, 8, 9)))                 pygame.key.ScancodeWrapper(7, 8, 9)
```

PyO3 kann `tuple` nicht erweitern — eine Unterklasse braucht eine feste
Größe, `tuple` hat eine veränderliche. Also steht die Klasse jetzt in
Python, trägt die 512 Zustände als Tupelinhalt, und ihr `__getitem__`
rechnet mit einem Rust-Helfer `key._platz` vom **Tastencode** auf den
**Tastenplatz** um — die Umrechnung, die pygames C-Code im Verborgenen
macht.

### Die Frage, die `dir()` nicht stellt: von was erbt eine Klasse?

`dir()` gegen `dir()` prüft die Namen. Es prüft **nicht** die
Vererbungskette — und daran hingen drei weitere Funde. Die Probe
`Klassenbasen` vergleicht jetzt die `__mro__` aller 65 Klassen, also genau
das, woran `isinstance` und `issubclass` hängen.

**`Sound` und `PixelArray` hatten eine Ebene zuviel.**

```text
pygame   Sound.__mro__[1:]   ('object',)
rustgame Sound.__mro__[1:]   ('Sound', 'object')
```

Dort stand je eine dünne Python-**Unterklasse**, die nichts beisteuerte
außer `__init__`, und die Begründung daneben war richtig: PyO3 0.26 kennt
keinen `tp_init`-Schlitz, also sind `Sound()` und `Sound.__new__(Sound)`
nicht zu unterscheiden; eine Unterklasse in Python hat den Schlitz von
selbst.

**Der Schluss daraus war falsch.** Eine PyO3-Klasse ist ein **Heap-Typ**,
und bei einem Heap-Typ setzt `setattr(Klasse, "__init__", f)` den Schlitz
mit: CPythons `type_setattro` ruft `fixup_slot_dispatchers`, und das legt
`tp_init` auf `slot_tp_init`. Nachgemessen — es geht. Beide Umwege sind
weg, und mit ihnen ein falsches `__module__` (`rustgame_py._sound` statt
`rustgame_py.mixer`).

Dabei fiel noch ein Fehler im eigenen Flicken auf: die Prüfung „hängt
`__init__` schon dran?" fragte nach dem **Namen**, und PyO3 trägt für seine
eigene Rust-Methode bereits einen `method descriptor` unter diesem Namen
ein. Also wurde nie etwas angehängt, und `Sound()` hörte auf zu meckern.
Gefragt wird jetzt nach `__code__` — das hat nur eine Python-Funktion.

**`camera.Camera` und `_camera.Camera` sind zwei Klassen.** Hier war es
eine. Gemessen:

```text
pygame   camera.Camera     _PreInitPlaceholderCamera(AbstractCamera)
pygame   _camera.Camera    Camera(object)
```

Vor `camera.init()` ist `Camera` ein **Platzhalter**, der bei jeder Frage
`pygame.error: pygame.camera is not initialized` wirft; `init()` tauscht
`Camera` und `list_cameras` gegen die echten aus, `quit()` setzt den
Platzhalter zurück. Der Platzhalter fehlte hier ganz — **die Kamera
arbeitete vor `init()`**, und keine Probe hat das gefragt. Jetzt ist
`pygame/camera.py` Zeile 45–74 übertragen, samt dem zweiten Zweig
(`NotImplementedError`, wenn das Modul läuft und die Namen trotzdem nicht
getauscht sind).

Hätte es diese Probe früher gegeben, wäre auch `key.ScancodeWrapper` nicht
durch Zufall aufgefallen: ein Nicht-Tupel, wo pygame ein `tuple` hat, ist
genau ein Unterschied in der ersten Zeile der `__mro__`.

### Der dreizehnte blinde Fleck: Überschuss wurde nie gefragt

`api_coverage.py` fragt, was pygame **hat**, und meldet, was fehlt. Was
rustgame **zuviel** hat, fällt dort durch — und am Hauptmodul standen zwei
Namen zuviel:

```text
hasattr(pygame, "Circle")   False   -  auch nach `import pygame.geometry`
hasattr(pygame, "Line")     False
```

Beide gibt es bei pygame nur als `pygame.geometry.Circle`. Die Probe
`Untermodule gesamt` vergleicht `dir()` schon in beide Richtungen — aber
nur für **Untermodule**; das Hauptmodul selbst war nie dabei. Jetzt
vergleicht `Hauptmodul Namen` seine 679 Namen in beide Richtungen.

Ein Name fiel dabei als Unterschied an, der keiner ist: **zwölf
Untermodule** (`camera`, `docs`, `freetype`, `midi`, `newbuffer`, `pypm`, …)
sind bei pygame erst nach `import pygame.X` ein Feld des Pakets, bei
rustgame von Anfang an. Das ist ein Unterschied des **Zeitpunkts**, nicht
der Schnittstelle. Statt zwölf Namen in eine Ausnahmeliste zu schreiben —
und damit den nächsten blinden Fleck anzulegen — lädt die Probe **alles**
auf beiden Seiten und vergleicht danach ohne Ausnahme. Ausgelassen wird
genau ein Name: `tests`, pygames eigene Testsuite.

### Und `from pygame.x import *` brachte etwas anderes

`dir()` beantwortet nicht, was `import *` holt: dafür zählt `__all__`, und
sonst das **Wörterbuch**. Gemessen hatten **51 von 60** Untermodulen hier
ein anderes `__all__` als pygame — PyO3 trägt jeden Namen, der mit
`PyModule::add` kommt, dort mit ein —, und bei **vierzehn** brachte
`import *` deshalb etwas anderes:

```text
camera    fehlt ABC, abstractmethod, os, platform, sys, warnings
cursors   fehlt pygame
_sprite   fehlt warnings, zuviel OrderedUpdates, Rect, from_surface, …
freetype  zuviel SysFont
ftfont    zuviel FontType
```

Die ersten drei waren Namen, die mit `setattr` kamen und damit **nicht** in
`__all__` landeten, im Wörterbuch aber standen. Die letzten zwei sind der
umgekehrte Fall: dort hat **pygame** ein eigenes `__all__`, und es ist
schmaler als sein Wörterbuch.

Genau **elf** Module geben sich bei pygame selbst ein `__all__`
(`constants`, `freetype`, `ftfont`, `locals`, `macosx`, `midi`, `pkgdata`,
`sndarray`, `surfarray`, `typing`, `version`); alle anderen und das
Hauptmodul haben keines. `lib.rs` setzt darum am Ende der Modulfunktion für
jedes Modul `__all__` auf die öffentlichen Namen, abzüglich dessen, was
pygames eigenes `__all__` ausläßt — mit der Messung je Modul daneben. Die
Probe `Sternimport` vergleicht alle 61, damit die Tabelle nicht veraltet.

Zwei Nachwehen hat dieser Umbau selbst erzeugt, und beide hat sofort eine
Probe gemeldet:

* **`pygame._sprite` reichte `__all__` weiter.** Sein `__getattr__` leitet
  alles an `sprite` durch — auch Doppelstrich-Namen. `import *` brachte
  damit 25 Namen statt der 19 im Wörterbuch. Jetzt bleiben Doppelstriche
  draußen.
* **`ftfont.__dir__` las `__all__`.** Sobald `lib.rs` es auf pygames
  schmalere Fassung zurechtschnitt, verschwand `FontType` aus `dir()`. Zwei
  Fragen brauchen zwei Listen: `_SICHTBAR` ist, was `dir()` zeigt,
  `__all__` ist, was `import *` bringt.

### Eine benannte Grenze, die gefallen ist

Elf Grenzen standen hier, und eine war keine.

> **`Surface()` ohne Größe.** PyO3 bildet `__init__` nicht auf `tp_init`
> ab, also sieht `Surface()` genau dasselbe wie
> `Surface.__new__(Surface)` — beide kommen nur durch `tp_new`, und dort
> ist nicht zu unterscheiden, ob danach noch ein `__init__` folgt. Eines
> von beiden muss falsch sein. […] **Und der Ausweg, der bei `PixelArray`
> und `ftfont` getragen hat, trägt hier nicht:** dort ist die Klasse eine
> Unterklasse **in Python**. Bei `Surface` wäre das falsch, denn die
> meisten Flächen entstehen in Rust und wären dann keine Instanzen der
> Python-Klasse mehr.

Der zweite Teil ist richtig. Der erste war falsch — und er stand hier
sechs Wochen, weil niemand die Annahme geprüft hat, auf der er ruht. Eine
PyO3-Klasse ist ein **Heap-Typ**, und bei einem Heap-Typ setzt
`setattr(Klasse, "__init__", f)` den Schlitz mit: CPythons
`type_setattro` ruft `fixup_slot_dispatchers`, und das legt `tp_init` auf
`slot_tp_init`. **Es braucht keine Unterklasse.**

Aufgefallen ist das nicht beim Nachdenken über `Surface`, sondern beim
Aufräumen von `Sound` und `PixelArray`: dort trug dieselbe Begründung je
eine Python-Unterklasse, und die Probe `Klassenbasen` hat sie als eine
Ebene zuviel gemeldet. *Eine Begründung, die an zwei Stellen dasselbe
sagt, ist an beiden zu prüfen.*

Jetzt ist die Aufteilung dieselbe wie bei pygame, Zeile für Zeile
gemessen:

```text
Surface()                             TypeError: function missing required
                                      argument 'size' (pos 1)
Surface.__new__(S)                    <Surface(Dead Display)>
Surface.__new__(S).get_size()         error: Surface is not initialized
Surface.__new__(S, (4,4)).get_size()  error: Surface is not initialized
Surface((4,4)).get_size()             (4, 4)
Surface(1)                            ValueError: size needs to be
                                      (number width, number height)
```

Die vierte Zeile ist die überraschende: pygames `tp_new` **sieht seine
Argumente gar nicht an**. Und die letzte war schon vorher falsch — hier
stand `TypeError: offset must be two numbers`, der Satz des
Versatz-Lesers. Richtig für einen Versatz, falsch für eine Größe, und
keine Probe hat ihn gelesen.

Zwei Nachwehen, beide sofort gemeldet:

* **Sechs Tests in `surface_test.py`** wurden rot: `copy`, `convert` und
  `convert_alpha` gaben auf einer **Unterklasse** eine Fläche, die
  `Surface is not initialized` meldet. `wie_typ` übernimmt die Felder des
  Neubaus einzeln und hatte `uneingerichtet` nicht dabei — vorher war es
  immer `false`, weil `__new__` schon gebaut hatte. **Keine der 804 Proben
  hat es gesehen**, denn sie fragen alle die Klasse selbst und keine
  Unterklasse.
* Als Zugabe geht jetzt, was vorher am eifrigen `__new__` scheiterte:
  `class Zwei(Surface): def __init__(self, b, h): super().__init__((b, h))`.

### Und eine Grenze, die längst keine mehr war

Elf Einträge standen in `BEKANNTE_GRENZEN`, der Lauf meldete **zehn**.
Eine davon löste gar nicht mehr aus:

> **`PixelArray` sperrt die Fläche.** pygame trägt ein frisches
> `PixelArray` als Sperre in seine Fläche ein (`get_locks()` enthält
> danach das PixelArray), hier bleibt die Liste leer. Grund ist eine
> Grenze von PyO3 […]

Gemessen geben beide Seiten `get_locks()` mit einem Eintrag, `get_locked()`
mit `True`, und der Eintrag **ist** das PixelArray. Die Grenze war mit der
Python-Hülle gefallen, die sie behob — und der Eintrag blieb stehen.

Das Werkzeug zählt nur, was auslöst, also stand die Zahl richtig da. Aber
eine Grenze, die nicht mehr eine ist, ist genau das, wovon der Abschnitt
„Vermerke veralten still" handelt. Jetzt ist sie weg, und die zehn in der
Tabelle sind die zehn, die auslösen — nachgezählt.

### Der Eingriff, der falsch war: `pygame.imageext`

Eine Messung von Hand sagte: `hasattr(pygame, "imageext")` ist `False`,
auch nach `import pygame.imageext`. Stimmt — pygames `__init__.py` holt
zwei Funktionen von dort und **löscht den Namen** wieder, und eine
`import`-Anweisung bindet ihn für ein Modul, das schon in `sys.modules`
steht, nicht neu. Also wurde das Feld hier weggelassen: ein Name weniger,
`dir()` deckungsgleich.

**Drei Tests in `imageext_test.py` wurden rot.** `from pygame import
imageext` findet bei pygame das Untermodul über `__path__`, wenn das Feld
fehlt — `rustgame_py` ist **ein einziges Modul** und hat kein `__path__`.
Ohne Feld greift der Test zum echten pygame und bekommt
`TypeError: argument 1 must be pygame.surface.Surface`.

Der Eingriff ist zurückgenommen, und das ist jetzt die zehnte Grenze: ein
Name mehr am Hauptmodul, mit Grund. Zwei Dinge sind daran zu lernen.

**Die stärkere Messung lag schon vor.** Die Probe `Hauptmodul Namen` hatte
„gleich" gemeldet — weil sie mit `importlib.import_module` lädt, nicht mit
der `import`-Anweisung. Korrigiert wurde nach der schwächeren von zwei
Messungen, und die Probe hat den Rückschritt beim nächsten Lauf sofort
gemeldet.

**Und `dir()` ist nicht das letzte Wort.** Hier stand sechsmal „ein Name
zuviel ist derselbe Fehler wie einer zuwenig", und das gilt — solange der
Name nichts trägt. Dieser trug: er ist der einzige Weg zu einem Untermodul
in einem Paket, das kein Paket ist.

### Die Frage, die alle Namensproben auslassen: die Doppelstriche

Jede Namensprobe hier filtert Namen mit Unterstrich — und ein
**Doppelstrich ist ein Name mit Unterstrich**. Damit fragte niemand nach
den Methoden, an denen die Sprache selbst hängt. Gemessen gingen **22 von
65** Klassen auseinander, und drei davon waren keine Kosmetik.

**`Vector2 += w` war kein `+=`.** Es fehlten `__iadd__`, `__isub__`,
`__imul__`, `__itruediv__` und `__ifloordiv__`; ohne die fällt Python auf
`v = v + w` zurück:

```text
v = Vector2(1, 2); i = id(v); v += Vector2(1, 1)
pygame    id(v) == i   True     -  derselbe Vektor
rustgame  id(v) == i   False    -  ein neuer
```

Das Ergebnis stimmte, das **Objekt** war ein neues — und wer den Vektor
woanders festhält (ein Sprite, eine Liste, ein zweiter Name), sieht die
Änderung nicht. Die fünf stehen jetzt in Python und werden per `setattr`
angehängt, denn PyO3 verlangt für `nb_inplace_*` den Rückgabetyp `()` und
kann damit **kein `NotImplemented`** liefern — genau das gibt pygame für
einen Operanden, der nicht passt (`v *= Vector2(2,2)` fällt dort auf das
**Skalarprodukt** zurück, und aus dem Vektor wird eine Zahl).

**`Cursor` war keine Folge.** `len(c)`, `c[0]` und `list(c)` gaben
`TypeError`, wo pygame 1, die Konstante und `[0]` gibt — ein system-Zeiger
hat Länge 1, ein Bitmap-Zeiger 4, ein Farbzeiger 2. pygames Umsetzung
reicht an sein Datentupel durch; deshalb stimmen jetzt auch die
Fehlertexte, ohne dass einer davon im Quelltext steht. Hier stand
stattdessen ein `__bool__`, das immer `true` gab — mit dem richtigen
Vermerk, `Cursor(0)` dürfe nicht `False` sein. Die Lösung war die falsche:
mit `__len__` kommt `bool()` von selbst richtig heraus.

**`system.PowerState` war kein `@dataclass`.** Es war eine `#[pyclass]`,
die pygames `repr` Wort für Wort nachbaute — und damit fehlten
`__match_args__` (also jedes `match`-Muster), `__dataclass_fields__` und
`__replace__`, während **acht Namen zuviel** waren: PyO3s `get_all` legt
die Felder als Klassenbeschreiber ab, `dir(pygame.system.PowerState)` hat
dagegen **keinen** öffentlichen Namen. Jetzt ist es pygames eigene
Fassung, Zeile für Zeile — möglich, weil `get_power_state()` hier immer
`None` gibt und die Klasse von Rust aus nie gebaut wird.

Dazu fünf kleinere Funde: `__safe_for_unpickling__` (bei `Rect` ein Feld,
bei `Vector2` eine **Methode** — dieselbe Marke, zwei Gestalten, beides
gemessen), `ScancodeWrapper.__next__`, `pypm.Input.__setstate__`,
`sprite.AbstractGroup.__class_getitem__` (ohne das ging
`GroupSingle[Sprite]` nicht) — und vier Namen zuviel:
`Sound.__deepcopy__`, `BufferProxy.__len__` (`len(s.get_buffer())` gibt
bei pygame `TypeError`, hier 16) und `Mask.__array_struct__` (pygames
Maske hat **keine** Feldschnittstelle; `numpy.array(maske)` gibt dort die
Form `(1, 4)`).

### Und zwei deutsche Namen in der öffentlichen Schnittstelle

`api_coverage.py` fragt, was pygame **hat**, und meldet Fehlendes. Ein
**Überschuss** fällt dort nie auf — und er war da:

```text
dir(pygame.bufferproxy.BufferProxy)   laenge, aus_woerterbuch
```

Zwei Hilfsrechnungen standen im `#[pymethods]`-Block, und **ein
`#[pymethods]`-Block ist die Schnittstelle**: wer dort eine Hilfe ablegt,
veröffentlicht sie. Sie liegen jetzt in einem gewöhnlichen `impl`. Dazu
hatte `camera.AbstractCamera` zwei abstrakte Methoden mehr als pygames und
war damit **strenger** als das Vorbild: ein eigenes Kamerastück ohne
Regler ließ sich nicht einmal anlegen.

Die Probe `Klassennamen` vergleicht darum die **rufbaren** Namen aller 65
Klassen in beide Richtungen. Nur die rufbaren, und das ist eine
Entscheidung mit Grund: PyO3 legt ein Feld (`#[pyo3(get, set)]`) als
Beschreiber an der **Klasse** ab, pygames Python-Klassen setzen dasselbe
Feld im `__init__` an der **Instanz**. Das betrifft sechzehn Klassen und
immer dieselbe Sorte Name — `sprite.Group.lostsprites`,
`DirtySprite.dirty`, `collide_rect_ratio.ratio`. An der Instanz hat sie
**beide** Seiten; an der Klasse nur rustgame. Eine fehlende oder
zusätzliche **Methode** bricht ein Programm, ein Beschreiber, den auch die
Instanz hat, nicht.

### 121 gegen 121 ist eine Zahl, keine Menge

Hier stand, pygames eigene Suite laufe mit „2364 grün, 0 Abweichungen, 121
übersprungen" und pygame selbst überspringe dieselben 121. Der zweite Teil
war eine **Zahl**, kein Namensvergleich — und 121 gegen 121 kann auch
heißen: rustgame überspringt Test A, pygame fährt ihn, und pygame
überspringt dafür Test B, den rustgame fährt. Die Summe stimmt, und ein
Test ist **unsichtbar** geworden: nicht rot, sondern gar nicht gelaufen.

Genau diese Sorte Fehler hat hier schon dreimal zugeschlagen — eine
Testdatei, die nicht in der Liste stand; eine, die auf Modulebene nicht
lud; fünfhundert Prüfungen, die der Läufer nicht fand.

`pygame_tests.py` hat jetzt `--uebersprungen` (nur die Kennungen, eine je
Zeile), und `tools/uebersprungen.py` fährt beide Engines und stellt die
Listen gegeneinander. Gemessen:

```text
pygame ueberspringt 121, rustgame 121
Dieselben 121 Tests, Name fuer Name.
```

In beide Richtungen leer. Die Behauptung ist damit eine Messung.

### Eine Grenze, die ihre eigene Behauptung prüfen lässt

Die Grenze `Felder an der Klasse` sagt: PyO3 legt ein Feld als Beschreiber
an der **Klasse** ab, pygame setzt dasselbe Feld im `__init__` an der
**Instanz** — und dann den Satz, auf den es ankommt:

> An der Instanz hat sie **beide** Seiten.

Das ist eine Behauptung, und Behauptungen sind zu messen. Die neue Probe
`Instanznamen` baut je eine Instanz und vergleicht ihre öffentlichen
Namen: **28 Klassen, 0 Unterschiede, keine, die sich nicht bauen ließ.**
Die Grenze ist damit nachgeprüft und nicht nur begründet.

Gebaut werden nur die, bei denen das Bauen nichts kostet — `_sdl2.Window`
öffnet ein Fenster, `AudioDevice` ein Tongerät, `Camera` sucht die Kamera.
Eine Probe, die Geräte anfasst, ist keine Probe mehr. Die Klassenebene
deckt dafür alle 65 ab (`Klassennamen`, `Klassendoppel`, `Klassenbasen`);
`Instanznamen` ist die Gegenprobe dazu.

### Und ein Weg, der keiner war: die Signaturen

Niemand vergleicht, wie viele Argumente eine Funktion nimmt. Der
naheliegende Weg ist `__text_signature__` — und er führt nirgendwohin.
Gemessen:

```text
pygame-Funktionen mit Signatur   131
            ohne                 262
pygame.get_error        ($self, /)
pygame.draw.rect        None
pygame.transform.scale  None
```

pygame verrät sie nur bei Funktionen **ohne Argumente**; alles
Interessante gibt `None`. PyO3 erzeugt für jede Funktion eine echte
Signatur, rustgame ist dort also auskunftsfreudiger — vergleichen lässt
sich nichts. Der andere Weg, jede Funktion mit falschen Argumenten zu
rufen und die Fehler zu vergleichen, scheidet aus: `display.quit()`,
`event.wait()` und `time.wait()` **tun** dabei etwas. Die Arität prüfen
darum die 810 Proben an den Stellen, an denen sie zählt, und nicht ein
Werkzeug im Gießkannenverfahren.

### Der Name unter dem Namen

Alle Namensproben vergleichen, unter welchem **Attributnamen** etwas im
Modul steht. Keine fragte, wie das Ding **selbst** heißt — und das sind
zwei verschiedene Dinge:

```text
pygame.event.post.__name__       post
rustgame_py.event.post.__name__  ereignis_post
```

`#[pyfunction]` nimmt den **Rust**-Namen, wenn kein `#[pyo3(name = …)]`
dabeisteht, und `als(m, "post", f)` setzt nur das Modulfeld. Nachgezählt
trugen **258 von 390** Funktionen so ihren Rust-Namen: `tf_flip`,
`ton_get_busy`, `taste_get_mods`, `bild_load`, `anzeige_flip`,
`maus_get_pos`, `gfx_aacircle`.

Sichtbar war das an drei Stellen:

* `repr(f)` und jeder Rückverfolgungsauszug nannten den falschen Namen,
* **jede** Fehlermeldung über ein unbekanntes Schlüsselwort nannte ihn mit
  — genau der Satz, der als Grenze `unbekanntes Schlüsselwort` vermerkt
  war (`tf_scale_by() got an unexpected keyword argument 'y'`). Dort stand
  sogar, ein `#[pyo3(name = …)]` brächte `scale_by()` statt
  `tf_scale_by()` — „näher, nicht gleich". Das war richtig und hat die 258
  falschen Namen trotzdem stehen lassen. **Näher ist besser, und „nicht
  gleich" ist kein Grund, es nicht zu tun.**
* ein Werkzeug, das Aufrufe mitschreibt, ordnet sie dem falschen Namen zu
  — daran ist `tools/abdeckung.py` beim ersten Lauf gescheitert.

Jetzt sind es **0**, und die Probe `Funktionsnamen` vergleicht `__name__`
jeder Funktion, jeder Klasse und jeder Methode sowie `__qualname__` der
Klassen. Zwei weitere Funde kamen dabei mit: `ftfont.Font` entsteht in
einer Funktion, und Python schreibt das mit
(`_font_klasse.<locals>.Font` statt `Font`); und `pygame.typing.Point` ist
`SequenceLike[float]`, nicht `Any` — sein `__name__` ist darum
`SequenceLike`.

### `display.Info` ist eine Fabrik, und `VidInfo` war unsichtbar

Beim Vergleich der Funktionsnamen fiel `display.Info` auf, weil es in der
Funktionsliste gar nicht auftauchte — hier war es eine **Klasse**:

```text
type(pygame.display.Info)   builtin_function_or_method
pygame.display.Info()       <class 'pygame.display.VidInfo'>
dir(pygame.display)         …Info…   -  **kein** VidInfo
```

Dasselbe Muster wie `joystick.Joystick`: eine Fabrik, die aussieht wie
eine Klasse. Und weil `VidInfo` **in keinem Modul steht**, hat sie keine
der Klassenproben je gesehen — `api_coverage.alle_klassen` zählt auf, was
in einem Modul steht, nicht was ein Aufruf zurückgibt.

Dabei kam eine Eigenheit zum Vorschein, die man nicht erfindet: pygames
`VidInfo` zeigt ihre Felder **nirgends** und sie arbeiten trotzdem.

```text
dir(type(pygame.display.Info()))                  []
[n for n in dir(pygame.display.Info()) if …]      []
pygame.display.Info().hw                          0
```

Sie benutzt den alten Schlitz `tp_getattr`, und der trägt keinen Namen in
das Wörterbuch des Typs. Mit `#[getter]` gibt es das nicht — hier standen
neunzehn, und sie standen damit in `dir()`. Mit **einem** `__getattr__`
schon: der Doppelstrich wird von jeder Namensprobe ausgelassen, die Felder
sind nur über ihn zu erreichen. Jetzt ist `dir()` auf beiden Seiten leer
und alle neunzehn Werte stimmen. Dazu zwei Fehlertexte, die vorher PyO3s
waren: `does not exist in vidinfo` und
`… has no attribute 'hw' and no __dict__ for setting new attributes`.

Die Probe `Instanznamen` vergleicht seitdem auch den **Typnamen** jeder
Instanz — damit sind Rückgabetypen nicht mehr unsichtbar.

### Die Frage hinter der Tabelle: wie viel davon läuft je?

`API.md` sagt `3471 von 3471 Namen (100 %)`. Das heißt: jeder Name, den
pygame hat, **existiert** hier auch. Es heißt **nicht**, dass irgendetwas
davon je gelaufen ist. Ein Name, der da ist und nie gerufen wurde, besteht
jede Messung dieses Projekts und bricht erst im Programm eines Menschen.

`tools/abdeckung.py` schreibt mit `sys.monitoring` jeden Aufruf mit —
`events.CALL` liefert auch für C- und PyO3-Funktionen das gerufene Objekt
— und fährt beide Quellen, aus denen hier Aufrufe kommen: die 810 Proben
und pygames Testsuite. Was danach nicht in der Menge steht, hat kein
Aufruf dieses Projekts je angefasst.

Die Zahl ist **keine** Zeilen-Abdeckung: eine Funktion, die einmal
gerufen wurde, zählt als abgedeckt, auch wenn nur einer ihrer Zweige lief.
Und sie zählt nur Rufbares — Konstanten werden gelesen, nicht gerufen, und
`sys.monitoring` kennt kein Ereignis für einen Attributzugriff. Sie
beantwortet genau eine Frage, und die hatte vorher keine Antwort.

**Zweimal hat das Werkzeug dabei sich selbst gemessen**, und beide Male
war der erste Bericht falsch:

| Bericht | der wirkliche Grund |
|---|---|
| `event.post` nie gerufen (ruft `event_test.py` 47 Mal) | die Funktion hieß `ereignis_post` — siehe oben, 258 falsche Namen |
| `Group.add` nie gerufen, und dasselbe in sechs Unterklassen | geerbte Methoden melden sich als `AbstractGroup.add`, der Erwartungsschlüssel stand als `Group.add` — dieselbe Methode dreizehnmal gezählt |
| 32 `Vector3`-Methoden nie gerufen, obwohl `math_test.py` sie 75 Mal ruft | **`umbiegen()` benennt die Module um** — während der Testsuite heißt `rustgame_py.math` `pygame.math`, und das Werkzeug verglich gegen `rustgame_py` |
| derselbe Bericht, zweiter Anlauf | der Paketname wurde **im Rückruf** gelesen, also erst *nach* der Umbenennung: `paket.__name__` ergab `pygame`, verglichen wurde `pygame` gegen `rustgame_py.math` |

*Ein Messgerät, das etwas Unerwartetes meldet, hat zwei Verdächtige — und
einer davon ist es selbst.*

### Dreimal geraten, einmal gemessen

Der dritte Fall ist der lehrreichste, weil ich ihn **dreimal falsch
erklärt** habe, bevor ich ihn gemessen habe:

| Vermutung | was daraus wurde |
|---|---|
| „Ein Rückruf, der wirft, nimmt dem Werkzeug still die Ereignisse weg" | **widerlegt** — ein `try` mit Zähler blieb bei null, die Zahl unverändert |
| „Die Reihenfolge der zwei Quellen" | half, aber **aus einem anderen Grund**; die Zahl stieg von 56 % auf 72 %, die `Vector3`-Methoden blieben alle unten |
| „Datei für Datei statt in einem Zug" | half ebenfalls, und auch das war nicht die Ursache |

Gefunden hat sie schließlich die **einfachste Frage, die vorher keiner
gestellt hatte**: *läuft die Überwachung überhaupt noch?*

```text
vorher       events=16  tool='wer'  zaehler=2
nach import  events=16  tool='wer'  zaehler=51678
nachher      events=16  tool='wer'  zaehler=264049
```

Sie lief die ganze Zeit, und der Rückruf hatte **264.049** Aufrufe
gesehen. Die Ereignisse waren nie verloren — sie wurden alle weggeworfen,
weil `pygame_tests.umbiegen()` jedes Modul umbenennt (fünf Tests in
`font_test.py` fragen `pygame_font.__name__ == "pygame.ftfont"` ab, also
*muss* es das). Der Name des Pakets wird jetzt **vor** dem Suitenlauf
festgehalten, und beide Schreibweisen gelten.

*Wer dreimal eine Ursache vermutet, ohne sie zu messen, hat dreimal
geraten.* Seitdem nennt das Werkzeug zu jedem Teillauf seinen
**Durchsatz**, nicht nur sein Ergebnis:

```text
   [suite:math] 196223 Aufrufe gemeldet, 121 Namen zugeordnet
```

Diese eine Zeile hätte drei Fehlversuche gespart — „nichts gemeldet" und
„alles weggeworfen" sehen im Ergebnis gleich aus und sind zwei völlig
verschiedene Fehler.

### Und ein Verlust, der wirklich einer ist

Dieselbe Zeile hat danach gezeigt, dass der Verdacht auf die Reihenfolge
**nicht** nur Einbildung war:

```text
   [suite]      158461 Aufrufe gemeldet,  28 Namen zugeordnet   (alle 50 Dateien)
   [suite:math] 196223 Aufrufe gemeldet, 121 Namen zugeordnet   (eine Datei)
```

Fünfzig Dateien melden **weniger** Aufrufe als eine einzige. Innerhalb
eines Suitenlaufs geht also wirklich etwas verloren, und wo, ist offen.
Deshalb bleibt die Testsuite in Einzelprozessen, einer je Datei — der Lauf
dauert Minuten statt Sekunden, und das ist der Preis dafür, dass die Zahl
etwas bedeutet. Der Vermerk im Quelltext sagt ausdrücklich, dass die
Ursache **offen** ist; ein Vermerk, der eine Ursache behauptet, die nie
gemessen wurde, ist schlimmer als keiner.

## Runde sechs: was sagt es, wenn man es **falsch** benutzt?

Jede Probe dieses Projekts fragte bis hierher, was eine Funktion tut, wenn
man sie richtig ruft. Keine fragte nach dem häufigsten Fall im Leben eines
Programms: dem Fehlgriff. Also wurde jede öffentliche Funktion und jede
Klasse mit **siebzehn** Argumenten gerufen — siebzehn nimmt keine, also
räumt jede auf, bevor sie etwas tut, und der Aufruf ist nebenwirkungsfrei.

508 Antworten als ständige Probe (eine Funktion ist ausgenommen:
`newbuffer.Py_buffer(1)` lässt **pygame** mit
„(pygame parachute) Segmentation Fault" abstürzen — *ein Messgerät, das
das Messobjekt umbringt, misst nichts*). Beim ersten Lauf waren 289
verschieden. Die meisten harmlos (pygames
`takes at most 4 arguments (17 given)` gegen PyO3s
`takes from 1 to 4 positional arguments but 17 were given` — dieselbe
Aussage in anderen Worten), aber darin steckten **zwölf echte Fehler**:

| Fund | was es war |
|---|---|
| `display.update` | `TypeError` statt `ValueError`, und „sequence of **rec**styles" statt „an iterable of rectstyles" — ein Tippfehler aus einer älteren pygame-Fassung, hier nie nachgemessen |
| `display.update` ohne `set_mode` | pygame prüft den **Zustand vor den Argumenten**; hier kam der Argumentfehler zuerst |
| `scrap.get/put/set_mode/init` | dasselbe, vier Mal — und `scrap.contains` tut es **nicht**, auch das gemessen |
| `math.clamp` + vier weitere | pygame schreibt eigene Sätze hin (`clamp requires 3 arguments`) |
| `mouse.set_pos` | *ein* Satz für jede falsche Zahl von Argumenten (`invalid position argument for set_pos`), nicht zwei verschiedene |
| `camera.AbstractCamera` | war keine echte abstrakte Klasse — `__init__` fehlte unter den sieben Pflichtmethoden |
| `typing.SequenceLike` | die beiden Methoden trugen kein `@abstractmethod`, `__abstractmethods__` war leer |
| `typing.ColorLike/FileLike/RectLike` | waren `Any` und damit **rufbar**; pygames sind Vereinigungstypen und lassen sich nicht rufen |
| `geometry.Circle` | der Satz war nach einem Drittel abgeschnitten |
| `_sdl2.error` | ist eine **eigene** `RuntimeError`-Klasse, nicht `pygame.error` — hier stand der Vermerk „also `pygame.error` selbst", und der war nie gemessen |
| `_sdl2.RendererDriverInfo` | nahm fünf Argumente; pygames nimmt **keins** und hat danach kein einziges Feld |
| `_render.Texture` | nahm sechs Argumente; pygames nimmt sieben (`scale_quality` fehlte) |
| `freetype.Font` | prüfte die **Zahl** erst nach dem Lesen der Werte und sagte darum `must be real number, not str` |
| `pkgdata.files` | war nachgebaut statt durchgereicht — pygame schreibt `from importlib.resources import files`, mit deren Signatur und deren `__module__` |
| `BufferProxy()`, `Mask()` | der Wortlaut nannte die Zahl nicht mit (`takes exactly one argument` statt `takes exactly 1 argument (0 given)`) |

### Der fünfzehnte blinde Fleck: die Namen der Argumente

Ein Nebenfund wog schwerer als die ganze Liste. Diese Fehlermeldung kam
aus **unserem** Modul:

```text
clamp() missing 3 required positional arguments: 'wert', 'min', and 'max'
```

Drei deutsche Namen in einer öffentlichen Schnittstelle — und
`clamp(wert=1, min=0, max=2)` **ging sogar**. pygames C-Funktionen nehmen
dort überhaupt keine Schlüsselwörter.

Nachgezählt nahmen **58 Funktionen** Schlüsselwörter an, die pygame
ablehnt, und einundzwanzig dieser Namen waren deutsch: `wert`, `typ`,
`daten`, `art`, `an`, `sichtbar`, `konstante`, `millisekunden`, `ziel`,
`quelle`, `a_von`, `a_bis`, `b_von`, `b_bis`, `woher`, `freq`, `o`, `func`,
`text`, `path`, `type_`. Dazu kam, dass einige **englische** Namen trotzdem
die falschen waren: `file` statt `filename`, `arg` statt `o`,
`color_space` statt `color`, `eventtype` statt `type`.

Das ist derselbe Fehler wie die 258 Rust-Funktionsnamen aus Runde fünf, nur
eine Ebene tiefer: **der Name eines Parameters ist Schnittstelle, keine
Geschmacksfrage.** Behoben ist er mit einem Schrägstrich in der Signatur
(`#[pyo3(signature = (value, min, max, /))]`) und einer Schattenbindung in
der ersten Zeile des Rumpfes:

```rust
#[pyo3(signature = (value, /))]
pub fn maus_set_visible(value: bool) -> bool {
    // Der oeffentliche Name ist pygames; der deutsche lebt im Rumpf.
    let sichtbar = value;
    mouse::set_visible(sichtbar)
}
```

So wird die Schnittstelle englisch, ohne dass ein einziger Rumpf angefasst
wird — und die Rümpfe dürfen deutsch bleiben, denn sie sind Dokumentation,
nicht Schnittstelle.

### Zwei Messgeräte, die beide nicht taugten

Die Frage „nimmt diese Funktion Schlüsselwörter?" ließ sich zweimal falsch
stellen, und beide Male sah die Antwort plausibel aus.

**Erster Versuch: ein erfundenes Schlüsselwort.** `f(zzz_gibt_es_nicht=1)`
scheitert immer, und pygames Wortlaut verrät die Antwort
(`takes no keyword arguments` gegen `got an unexpected keyword argument`).
Das meldete **91** Funktionen — darunter 33, die schon nur nach Stellung
nehmen: **PyO3 antwortet auf einen unbekannten Namen immer gleich**, gleich
wie die Signatur aussieht. Das Messgerät konnte die beiden Fälle nicht
trennen.

**Zweiter Versuch: der eigene Name.** `f(**{erster_parameter: x})` trennt
sie — aber dann beantworten die zwei Hüllen **zwei verschiedene Fragen**:
pygame wird nach `parent` gefragt und rustgame nach `woher`, und beide
sagen „nimmt an". Zwei richtige Messungen, die zusammen „gleich" ergeben.

Die Frage muss von **außen** kommen. Sie kommt jetzt aus pygames
`.pyi`-Dateien: dort steht, wie jeder Parameter heißt, und beide Hüllen
bekommen dasselbe Schlüsselwort vorgelegt. Drei Antworten sind zu
unterscheiden:

| Antwort | heißt |
|---|---|
| `nimmt an` | der Name geht als Schlüsselwort durch |
| `lehnt ab` | nur nach Stellung — pygames C-Weg |
| `kennt den Namen nicht` | die Funktion hat einen **anderen** Namen |

Damit fängt **eine** Probe beides: den falschen Namen und das zuviel
angebotene Schlüsselwort. 213 Fragen, 0 Abweichungen.

### Eine Probe, die nichts misst, ist immer gleich

Beim ersten Lauf meldete diese Probe „gleich", und zwar aus dem
schlechtesten Grund: sie hatte **null** Einträge. `io.open` warf einen
`NameError` — das Modul führt kein `io` —, der im `except` der Stub-Suche
still verschwand, und zwei leere Tupel sind nun einmal gleich.

Beide neuen Proben haben seitdem eine **Untergrenze**:

```python
if len(aus) < 150:
    raise RuntimeError(
        "Schluesselwoerter: nur %d Eintraege - die Stubs fehlen" % len(aus))
```

*Ein grüner Lauf ist kein Beweis, solange nicht geprüft ist, dass
überhaupt gemessen wurde.*

### Wie die Stelligkeit vergleichbar wird

Den Fehlertext wörtlich zu vergleichen wäre falsch gewesen — die beiden
Argumentleser schreiben dieselbe Aussage verschieden. Aus beiden Formen
wird darum die **Zahl** gelesen und die Zahl verglichen:

```text
pygame    function takes at most 4 arguments (17 given)        -> 4
rustgame  Surface._init() takes from 1 to 4 positional ...      -> 4
```

Drei Feinheiten entschieden darüber, ob das trägt:

* **Nur das Maximum.** pygames `PyArg_ParseTuple` prüft die Anzahl,
  *bevor* es die Pflichtargumente prüft, und meldet darum
  `takes at most 9` auch dort, wo vier Pflicht sind. Der Mindestwert steht
  in keinem der beiden Texte verlässlich.
* **Die Klammer sagt, ob `self` mitzählt.** Siebzehn Argumente wurden
  übergeben; steht dort 18, zählt die Meldung `self` mit. PyO3s `__new__`
  zählt `cls` **nicht** mit, ein Python-`__init__` schon — geraten werden
  musste das nicht.
* **Ist pygames Text keine Zählmeldung, ist er handgeschrieben** — und
  dann wird er wortgleich verglichen. Das ist die Regel, mit der
  `clamp requires 3 arguments` gefunden wurde, **ohne** eine Liste von
  Hand zu pflegen.

### `draw.aaline`: pygame widerspricht sich selbst

Eine Zahl ließ sich nicht angleichen, weil pygame zwei verschiedene nennt:

```text
aaline(s, c, (0,0), (9,9), 3, 1, 1)   takes at most 6 arguments (7 given)
aaline(s, c, (0,0), (9,9), 3, 1)      takes at most 5 positional arguments (6 given)
```

Die sechste Stelle ist das abgekündigte `blend`, das nur noch als
Schlüsselwort durchgeht — `aaline(…, blend=1)` nimmt pygame an (mit
`DeprecationWarning`), als sechstes Stellungsargument nicht. Hier sind es
fünf, und das ist die Wahrheit über die Funktion; `blend=1` geht ebenso.
Eine falsche Zahl nachzuschreiben, um eine Probe grün zu bekommen, wäre
das Gegenteil von messen — also steht sie als benannte Grenze.

### Fünf kleine Fehler in meinem eigenen Werkzeug

Der Eingriff an 58 Funktionen lief über ein Skript, und das Skript irrte
fünf Mal — jedes Mal auf eine Art, die es wert ist, aufgeschrieben zu
werden:

| Fehlgriff | Folge |
|---|---|
| `nam.lstrip("mut ")` | `lstrip` nimmt eine **Zeichenmenge**, nicht ein Präfix: aus `typ` wurde `yp`, und die Umbenennung fand den Parameter nicht |
| `s.index("pub fn ablage_get")` | traf `ablage_get_init`, das zwanzig Zeilen früher steht — ein Präfix ist kein Name |
| Suche am **blossen** Namen | `wait` gibt es zweimal: `time.wait` nimmt keine Schlüsselwörter, `event.wait` schon. `event.wait(timeout=5)` war danach kaputt |
| `re.sub` über **alle** Signaturen einer Datei | benannte auch `event.get` und `event.peek` um, die im Stub `eventtype` heißen |
| Kommasplitten der Parameterliste | `Bound<'py, PyAny>` enthält ein Komma — die Liste zerfiel mitten im Typ |

Vier davon fand der Übersetzer sofort. Den dritten fand erst die Probe —
*ein Werkzeug, das viele Stellen anfasst, braucht eine Messung, die alle
anfasst.*

### Was die Abdeckung jetzt sagt

Mit beiden Fehlern behoben liefert `tools/abdeckung.py` eine Zahl, die
etwas bedeutet:

```text
921 rufbare Namen in der Schnittstelle
   Proben: 821 Namen
   rect     98   color 63   math 121   mask 84   ...
gerufen: 773 von 921 (84 %)
**Nie gerufen** - 148 Namen, nach Ort
```

Die 818 Proben allein erreichen **821** Namen; pygames Testsuite legt 37
dazu (mehr als die 921 der Schnittstelle, weil die Proben auch Namen mit
Unterstrich und fremde Helfer anfassen). Der Nenner ist von 961 auf **921** gefallen, und das ist eine
Berichtigung am Werkzeug: `sysfont.get_fonts`, `font.get_fonts`,
`freetype.get_fonts` und `ftfont.get_fonts` sind **dieselbe** Funktion,
und der erste Entwurf zählte jeden Weg dorthin einzeln. `rustgame_py.
sysfont` stand mit acht nie gerufenen Namen im Bericht, obwohl drei davon
unter einem anderen Weg längst gerufen waren — dieselbe Falle wie bei den
geerbten Methoden, die hier schon über `__qualname__` gelöst war. Der
Schlüssel ist jetzt die **eigene Heimat** einer Funktion, nicht der Pfad,
unter dem man sie findet. Dass der Zugewinn klein ist, ist kein Mangel der Suite, sondern das
Ziel der Proben: sie sind gebaut, um **breit** zu fassen, während die
Suite tief prüft.

Die Liste der nie gerufenen Namen ist jetzt lesbar — und sie teilt sich
klar in zwei Hälften:

| Ort | Namen | erreichbar? |
|---|---:|---|
| `joystick.JoystickType` | 20 | **nein** — kein Gamepad am Rechner |
| `_sdl2.controller_old.Controller` | 13 | nein — dito |
| `midi.Output` / `Input` | 10 / 4 | nein — kein MIDI-Gerät |
| `Window` (`focus`, `maximize`, …) | 9 | nein — kein Fenstermanager |
| `camera.AbstractCamera` | 7 | nein — abstrakte Klasse |
| `pypm.Input` / `Output` | 6 / 6 | nein — kein PortMidi |
| `rustgame_py.sysfont` | 5 | **nein** — `basename`, `dirname`, `join`, `splitext` sind `os.path`, nicht unsere |
| `_camera.Camera` | 4 | nur mit Kamera |
| `_sdl2.Renderer`, `AudioDevice` | 4 / 3 | nein — Fenster bzw. Rückruf aus dem Tonfaden |

**Keine fett gesetzte Zeile mehr.** Nach Runde acht ist die Liste
vollständig gerätegebunden: Gamepad, MIDI-Gerät, Fenstermanager, Kamera,
PortMidi, ein Rückruf aus dem Tonfaden — plus fünf Namen, die `os.path`
gehören und nur deshalb in `dir(sysfont)` stehen, weil pygames
`sysfont.py` sie dort einführt.

Dahin gekommen ist die Liste in drei Runden, und **jede neue Probe hat
beim ersten Lauf etwas gefunden**: Runde sechs `_freetype`, `sysfont`,
`Vector2`/`Vector3` und zwanzig Einzelnamen aus `cursors`, `imageext`,
`newbuffer`, `version`; Runde sieben die 86 deutschen Fehlertexte; Runde
acht zwei Formeln an der leeren Zeile und zwei Sätze am `Channel`. **Eine
Funktion, die nie gerufen wurde, ist keine geprüfte Funktion** — und das
war hier jedes Mal wörtlich zu nehmen.

### Die nächste Frage: was zeigt `help()`?

Eine Stelle ist nach dieser Runde gemessen und **nicht** angeglichen, und
sie steht hier, damit sie nicht in Vergessenheit gerät. pygames
`__doc__` beginnt mit der Signatur und einem Satz:

```text
pygame.draw.aaline.__doc__
  'aaline(surface, color, start_pos, end_pos, width=1) -> Rect
   Draw a straight antialiased line.'
```

Unseres beginnt mit deutscher Prosa über die Umsetzung. Die **Signatur**
zeigt `help()` auf beiden Seiten (Python baut sie bei uns aus
`__text_signature__`), der **Satz** daneben nicht. Das ist eine
Entscheidung, keine Lücke: die Texte hier erklären, *warum* etwas so ist,
und sind an vielen Stellen das Einzige, was eine Messung von einer
Vermutung unterscheidet. Sie gegen pygames Einzeiler zu tauschen hieße,
die teuerste Arbeit dieses Projekts wegzuwerfen.

Offen ist damit die Frage, ob der **erste** Satz zusätzlich pygames
Wortlaut tragen sollte — 3471 Mal, maschinell aus pygames Doku erzeugt.
Gemessen ist sie noch nicht; aufgeschrieben jetzt schon.

## Runde sieben: deutsche Sätze in Ausnahmen

Die sechste Runde rief jede Funktion mit **siebzehn** Argumenten und prüfte
damit die *Zahl*. Die siebte ruft mit **einem** Argument der falschen
*Sorte* — ein anderer Weg durch denselben Argumentleser, und er hat etwas
gefunden, das die Zahl nicht zeigen konnte:

```text
rustgame_py.mixer.music.load("keinklang.txt")
  pygame.error: Unbekanntes Klangformat - erwartet wird WAV, OGG,
                FLAC oder MP3
```

**Deutsch. In einer Ausnahme, die ein Mensch zu sehen bekommt.**
Nachgezählt waren es **86 Sätze** in den Kern-Kisten und der Hülle:

| Ort | Sätze | Beispiel |
|---|---:|---|
| `rustgame-core/src/formate.rs` | 32 | `TGA: Bildart {art} kennt rustgame nicht` |
| `rustgame-display/src/backend/` | 25 | `dieses Stueck kennt kein Vollbild` |
| `rustgame-mixer/src/` | 19 | `Unbekanntes Klangformat - erwartet wird …` |
| `rustgame-core/src/image.rs` | 4 | `PNG braucht Breite und Hoehe groesser null` |
| `rustgame-py/src/` | 4 | `Font() erwartet einen Dateinamen, ein Dateiobjekt oder None` |

Alle 86 sind jetzt englisch, in pygames Tonfall — 89 Stellen im Quelltext,
samt der Tests und Vermerke, die sie zitieren. **Die Kommentare und
Dokumentationstexte bleiben deutsch:** die sind Dokumentation, nicht
Schnittstelle. Der Unterschied ist genau der, der in Runde fünf die 258
Rust-Funktionsnamen und in Runde sechs die 58 deutschen
Schlüsselwortnamen betroffen hat — **was nach außen sichtbar ist, gehört
pygame; was nach innen zeigt, gehört dem, der es liest.**

Dasselbe noch zweimal, aus anderer Quelle:

* **Das Betriebssystem.** `image.load("gibtsnicht.png")` gab
  `FileNotFoundError: Das System kann die angegebene Datei nicht finden`
  — Windows' eigener Satz, in der Sprache des Systems. pygame schreibt
  dort seinen: `No file 'gibtsnicht.png' found in working directory
  '…'.` `pygame.mixer.Sound` machte es schon richtig, die fünf
  Bildlader nicht.
* **Die Schriftenliste.** Zwei von 391 Namen hießen hier
  `lucidasanstypewriterschrg` statt `…schräg`: die Tabelle kommt über
  `reg query`, dessen Ausgabe in der **Konsolen-Codepage** steht, und
  `from_utf8_lossy` macht aus dem `ä` ein Ersatzzeichen, das
  `_simplename` als nicht-alphanumerisch wegwirft. Der Versuch, das mit
  `cmd /c chcp 65001 & reg query …` zu beheben, hat die Liste von 391 auf
  **einen** Eintrag verkürzt — also zurückgenommen und benannt. *Ein
  Fehler, den man gegen einen größeren tauscht, ist kein behobener
  Fehler.*

### Ein echter Fehler in pygame-ce 2.5.7

Die Probe `Vektor Reste` hat etwas gefunden, das nicht hier liegt:

```text
Vector3(1, 2, 3), ein halbes Bogenmaß
  rotate_x_rad      (1.0,       0.316889, 3.591599)
  rotate_y_rad      (2.315859,  2.0,      2.153322)
  rotate_z_rad      (-0.081269, 2.234591, 3.0)
  rotate_x_ip_rad   (1.0,       0.316889, 3.591599)
  rotate_y_ip_rad   (1.0,       0.316889, 3.591599)   ← um x!
  rotate_z_ip_rad   (1.0,       0.316889, 3.591599)   ← um x!
```

**`rotate_y_ip_rad` und `rotate_z_ip_rad` drehen in pygame um die
x-Achse.** Die neun anderen Varianten stimmen. Die Warnung daneben
verrät, wie es passiert ist: `vector3_rotate_y_rad_ip() now has all the
functionality of vector3_rotate_y_ip_rad()` — beim Zusammenlegen der alten
und neuen Namen ist die Verdrahtung verrutscht.

Hier drehen sie um y und z. **Einen falschen Drehsinn nachzubauen wäre
schlimmer als jede Abweichung:** ein Programm bekäme stillschweigend
Unsinn, und der Fehler stammte nicht einmal von hier.

### Was eine Fehlermeldung zusagt — und was nicht

616 Funktionen und Methoden, jede mit einem Argument der falschen Sorte
gerufen. Beim ersten Lauf waren **189** Antworten verschieden, und die
Hälfte davon war kein Mangel, sondern **der Unterschied zweier
Argumentleser**:

```text
pygame    function takes at most 4 arguments (17 given)
rustgame  Surface._init() takes from 1 to 4 positional arguments but 17 were given

pygame    argument 1 must be pygame.surface.Surface, not str
rustgame  rect() missing 2 required positional arguments: 'color' and 'rect'
```

Beides ist derselbe Einwand, und beide Sätze stammen nicht von pygame,
sondern von seinem Werkzeug — `PyArg_ParseTuple` in C gegen PyO3s
erzeugten Vorbau. Die Probe erkennt sie an ihrer **Form** und macht aus
jedem die Marke `<argumentfehler>`; verglichen wird dann

* die **Art** der Ausnahme immer (sie entscheidet, ob ein `except` den
  Fehler fängt — das Einzige an einer Meldung, auf das sich ein Programm
  verlassen darf),
* der **Wortlaut** überall, wo pygame ihn selbst geschrieben hat.

Die Erkennung ist eine **Regel, keine Liste** — deshalb fällt ein neuer
handgeschriebener Satz weiterhin auf. Von 189 Abweichungen sind damit
**4** geblieben, alle vier mit Grund.

### Die vierzig Sätze, die jemand geschrieben hatte

| Fund | was es war |
|---|---|
| `Surface.set_clip`, `display.update` | `TypeError` statt `ValueError` — ein `except ValueError` sah den Fehler **gar nicht** |
| `Vector.dot`, `Vector3.cross`, `angle_to` | `ValueError: Sequence has the wrong length.` statt pygames `TypeError` mit eigenem Satz |
| `Mask.get_at/set_at/scale`, `display.set_mode` | **ein** Helfer sagte überall `offset`; pygame sagt `pos`, `scale`, `size` |
| `Surface.get_rect`, `get_frect`, `Mask.get_rect` | `only accepts keyword arguments` — bei der Maske `supports`, und das ist pygames eigene Inkonsequenz |
| `time.delay`, `time.wait`, `mixer_music.set_pos`, `Surface.unmap_rgb`, `set_alpha`, `Mask.to_surface` | pygame hat für jede einen eigenen Satz |
| `mouse.set_visible`, `mixer.find_channel` | pygame liest dort eine **Zahl**, nicht einen Wahrheitswert |
| sieben Schalter (`get_sdl_version`, `set_grab`, `get_pos`, …) | pygame nimmt **jeden** Wert als Wahrheitswert (`PyObject_IsTrue`); PyO3 verlangte einen echten und nannte `'PyBool'` |
| `display.set_icon`, `event.post`, `set_window_position`, `set_palette` | PyO3 nannte Rust-Typnamen: `'PyTuple'`, `'Event'`, ``Can't extract `str` to `Vec``` |
| `Color.set_length` | eine Nicht-Zahl wurde zu `i32::MAX` **verschluckt** und als Bereichsfehler gemeldet |
| `Surface.convert_alpha` | die Vorlage wurde nur auf `None` geprüft — `convert_alpha("x")` kam stillschweigend durch |
| `camera.init` | jeder Hintergrundname wurde angenommen; pygame kennt `unrecognized backend name` |
| `mixer_music.get_metadata` | gab für eine Datei, die kein Tonformat ist, leere Etiketten zurück |
| `pypm`, `_sdl2.init_subsystem` | Pythons `int()`-Fehler statt pygames `an integer is required` |
| `Surface.set_palette` | pygames Satz endet mit einem **Zeilenumbruch** |

### Die 51 Argumentnamen, und was davon zu beheben war

```text
pygame    TypeError: must be real number, not str
rustgame  TypeError: argument 'winkel': must be real number, not str
```

PyO3 hängt den **Namen des Parameters** vor jeden Umwandlungsfehler, und
an 51 Stellen war dieser Name deutsch (`grad`, `winkel`, `laenge`) oder
trug Rusts Rohbezeichner samt Raute (`r#type`). Die Namen sind jetzt
pygames (`angle`, `length`, `type`); der Vorsatz selbst bleibt und ist
eine benannte Grenze — ihn loszuwerden heißt, jede dieser Funktionen auf
`&Bound<PyAny>` umzubauen, und das zieht bei zwanzig Methoden eine Kette
von Rückgabetypen nach sich. Ein Versuch dazu scheiterte an genau dieser
Kette; die Rücknahme war die bessere Entscheidung.

Zwei Stellen waren es wert: `event.event_name` (dort stand `r#type` mit
Raute im Text) und die fünf `math`-Funktionen aus Runde sechs.

### Vier Fehlgriffe in meinen eigenen Messgeräten

| Fehlgriff | Folge |
|---|---|
| Markierungstext mit `\x00` | pygame lehnt ein eingebettetes NUL **eigens** ab (`ValueError: embedded null character`) — neun Messungen waren Artefakte des Messgeräts |
| Paketnamen nicht normalisiert | 109 Scheintreffer von 276: `rustgame_py.display.flip` gegen `pygame.display.flip` |
| nur `pygame` normalisiert, nicht beide | pygames `pygame.midi not initialised.` wurde zu `@.midi`, unser wortgleicher Satz blieb `pygame.midi` |
| Prüfsumme über alle 391 Schriftnamen | zu streng: die Liste ist eine Tatsache des **Systems**, nicht der Hülle |

Dazu ein Rückschritt, den eine Probe gefangen hat: beim Umbau der sieben
Schalter auf `&Bound<PyAny>` sind die **Vorgabewerte** verloren gegangen
(`signature = (linked=true)` übersetzt mit `&Bound` nicht), und
`version.get_sdl_version()` ohne Argument brach. Gemeldet hat es
`print_debug_info` — eine Funktion drei Module weiter, die es aufruft.
*Eine Probe, die Aufrufe wirklich ausführt, findet Dinge, die keine
Namensliste sieht.*

## Runde acht: die leere Zeile

Nach Runde sieben standen noch zwei Posten auf der eigenen Liste:
`ftfont.Font` mit **16** nie gerufenen Namen und drei `sprite`-Gruppen mit
je sechs. Beide sind jetzt Proben — und die erste hat zweimal etwas
gefunden, das keine Rasterkante ist.

### Zwei Formeln, nicht zwei Rasterungen

`pygame.ftfont` ist pygames FreeType-Ersatz für `pygame.font`.
Seine Glyphenmaße weichen hier ab, weil FreeType die Kontur aufs
Pixelgitter einrastet und fontdue nicht — eine benannte Grenze, und sie
noch einmal zu messen bringt nichts. Die Probe vergleicht darum
**Beziehungen** statt Zahlen:

```text
get_linesize() == get_height() - 1
size(t)[1]     == get_height()
render("").get_height() == get_linesize()
get_descent()  <  0  <  get_ascent()
```

Die letzte Zeile hielt. Die dritte nicht:

| | pygame | hier (vorher) |
|---|---|---|
| `freetype.get_rect("")` | `(0, 20, 0, 25)` — `asc − desc` | `(0, 20, 0, 26)` — die Zeilenhöhe |
| `freetype.render("")` | `26` — `get_sized_height()` | `27` — `get_linesize()` der Schriftkiste |

**Bei leerem Text gibt es nichts zu rastern.** Ein Unterschied dort kann
also keine Rasterkante sein, sondern nur eine andere Rechnung — und genau
darum ist er wertvoll: ein Zahlenvergleich hätte ihn unter „Glyphenbox,
höchstens eins daneben" begraben. Eine Beziehung tut das nicht.

Der zweite Fund ist der unangenehmere: `render("")` nahm
`get_linesize()` aus der Schriftkiste, `get_sized_height()` meldet
`ft_height()` nach Python — **zwei Zahlen in der eigenen Hülle, die
dasselbe heißen und um eins auseinandergehen.**

### Eine Abkürzung, die eine Fallunterscheidung mitnahm

Der erste Versuch setzte den Leerfall ganz nach vorn in `kasten()`:

```rust
if text.is_empty() && !self.senkrecht {
    return Ok(Kasten { h: (asc - desc) as u32, .. });
}
```

Damit stimmte `freetype.get_rect("")` — und `ftfont` brach. pygames
`ftfont.Font` setzt `pad = True`, und gepolstert ist die Höhe für
**jeden** Text dieselbe (`get_rect("")` und `get_rect("Hi")` geben beide
19). Den Zweig für leeren Text gab es längst, drei Zeilen weiter unten,
samt Polsterung — die Abkürzung führte daran vorbei.

*Eine Abkürzung, die an einer Fallunterscheidung vorbeiführt, nimmt sie
mit.* Richtig war, in dem vorhandenen Zweig **eine Zahl** zu ändern.

### Der Wächter hatte selbst einen blinden Fleck

`tools/deutsche_fehlertexte.py` meldete **0** — und zwei Zeilen später
gab ein Aufruf das:

```text
rustgame_py.error: freesansbold.ttf nicht gefunden -
                   RUSTGAME_DEFAULT_FONT setzen
```

Der Sucher kannte drei Gestalten: `Fehler::X("…")`, `new_err("…")` und
`raise X("…")` — also die Stellen, an denen ein Fehler **gebaut** wird.
Ein Fehlertyp mit eigenem `impl Display` schreibt seinen Satz aber erst
beim **Anzeigen** hin, mit `write!(f, "…")`, und das war die vierte
Gestalt. Drei weitere deutsche Sätze kamen so heraus.

*Ein Sucher, der Gestalten aufzählt, findet nur die, die jemand
aufgezählt hat* — dieselbe Falle wie bei jedem blinden Fleck in diesem
Projekt, nur diesmal im Messgerät selbst. Es ist der fünfte in dieser
Reihe und der erste, der eine **Null** vorgetäuscht hat.

### Die Sprite-Gruppen waren richtig — jetzt ist es gemessen

`GroupSingle`, `LayeredUpdates` und `LayeredDirty` sind wörtlich aus
`pygame/sprite.py` übernommen, und die neue Probe fand beim ersten Lauf
**keine** Abweichung: Ebenen, Reihenfolge, `draw`-Rechtecke,
`set_timing_threshold` samt pygames Tippfehler-Alias
`set_timing_treshold`, die beiden `collide_*_ratio`-Fabriken — alles
gleich. Das ist der Unterschied zwischen „stimmt wahrscheinlich, es ist
ja übernommen" und **stimmt, nachgezählt an 40 Werten.**

### Der letzte erreichbare Posten: `mixer.Channel`

Nach `ftfont` und den Sprite-Gruppen blieb ein Cluster, der ohne Gerät
läuft und nie gelaufen war: `Channel.fadeout`, `pause`, `unpause`,
`queue`, `get_queue`. Die Probe baut einen Klang **aus Bytes** (kein
Dateizugriff, auf beiden Seiten derselbe RIFF-Kopf) und vergleicht den
Zustand, nicht die Zeit — was `get_queue()` vor und nach einem `queue()`
sagt, ob `pause()` die Beschäftigung ändert, was ein `fadeout` auf einem
stillen Kanal tut.

Zwei Funde, beide derselbe alte Bekannte:

| | pygame | hier (vorher) |
|---|---|---|
| `Channel.queue("x")` | `The argument must be an instance of Sound` | `argument 'sound': 'str' object cannot be converted to 'Sound'` |
| `Channel.fadeout("x")` | `'str' object cannot be interpreted as an integer` | derselbe Satz mit `argument 'time': ` davor |

Und einer in der Probe selbst: sie verglich zuerst die Objekte, die
`get_sound()` und `get_queue()` zurückgeben — deren `repr` trägt die
Speicheradresse, und die ist bei jedem Lauf eine andere. Gefragt ist, **ob**
noch etwas hängt, nicht welches Objekt.

### Zwei Fehler im Abdeckungswerkzeug, und wie viel sie verdeckt haben

Die Liste der nie gerufenen Namen nannte `ftfont.Font` mit sechzehn
Einträgen — **obwohl eine Probe sie alle ruft.** Zwei Ursachen, beide im
Messgerät:

| Fehlgriff | Folge |
|---|---|
| Der Schlüssel war der **Weg** zur Funktion, nicht ihre Heimat | `sysfont.get_fonts`, `font.get_fonts`, `freetype.get_fonts` und `ftfont.get_fonts` sind **dieselbe** Funktion und wurden vierfach gezählt — 40 Phantomnamen im Nenner |
| Der Modulzweig griff **vor** dem `__qualname__` | aus `_font_klasse.<locals>.Font.get_bold` wurde `rustgame_py.ftfont.get_bold`, und das steht in keiner Erwartungsliste — sechzehn Namen galten als nie gerufen |

Beides korrigiert hebt die Zahl von **73 %** (700 von 961) auf **84 %**
(773 von 921), ohne dass eine einzige Zeile Bibliothekscode dazukam. *Die
Hälfte eines Messwerts ist das Messgerät* — und die Liste, auf der dieses
Projekt seine nächsten Schritte plant, war an zwei Stellen falsch.

## Runde neun: was geht mit einem Objekt, außer seine Methoden zu rufen?

Acht Runden haben die **Funktionen** gemessen — Namen, Stelligkeit,
Schlüsselwörter, Fehlertexte, ob sie je gerufen wurden. Keine hat gefragt,
was mit den **Objekten** geht:

```text
hash(r)                        und damit {r: 1} und set([r])
a == b, a < b, bool(a), len(a), list(a), 1 in a
copy.copy, copy.deepcopy, pickle.dumps
o.feld = x                     schreibbar oder nicht, und welcher Fehler
```

Jedes davon trifft ein Programm, das die Bibliothek benutzt, **ohne eine
einzige Methode falsch zu rufen**: ein `Rect` in einer Menge, ein
`deepcopy` des Spielzustands, ein `pickle` beim Speichern.

281 Messungen an zwölf Klassen, **fünfzehn Abweichungen** — und zehn davon
an derselben Stelle:

| `pygame.cursors.Cursor` | pygame | hier (vorher) |
|---|---|---|
| `hash(c)` | geht | `TypeError: unhashable type` |
| `{c: 1}`, `set([c])` | geht | dito |
| `copy.deepcopy(c)` | geht | `cannot pickle` |
| `pickle.dumps(c)` | geht | dito |
| `c.eigenes_feld = 1` | geht | `AttributeError` |

pygames `Cursor` ist eine **gewöhnliche Python-Klasse**, und `type` und
`data` liegen als Fächer in ihrem `__dict__` — daraus folgt all das von
selbst. Eine PyO3-Klasse hat nichts davon: mit `__eq__` und ohne
`__hash__` ist sie unhashbar (genau wie eine Python-Klasse es wäre), und
ohne `__reduce__` lässt sie sich nicht einfrieren. Vier Zeilen Rust —
`dict`, `__hash__`, `__reduce__`, `__deepcopy__` — und alle zehn stimmen.

**Das ist die Sorte Lücke, die keine Namensliste und keine Testsuite
findet:** `api_coverage` zählt `Cursor` als vorhanden, `cursors_test.py`
ruft seine Methoden, `dir()` stimmt auf den Namen — und `copy.deepcopy`
war trotzdem ein `TypeError`.

### pygame ist mit sich uneinig: `Circle` gegen `Line`

```text
Circle(1,2,3) == Circle(1,2,3)     True     hash(circle)  TypeError
Line(1,2,3,4) == Line(1,2,3,4)     False    hash(line)    geht
```

pygames `Line` hat **kein** `__eq__` — zwei Strecken mit denselben vier
Zahlen sind dort verschieden, und weil die Gleichheit die der Identität
ist, lässt sich eine Strecke hashen. Sein `Circle` macht es genau
umgekehrt. Hier folgt `Line` dem `Circle`: Wertgleichheit, und damit
unhashbar. Von zwei Antworten, die pygame selbst gibt, ist das die
brauchbare — und sie steht als benannte Grenze mit beiden Messungen.

### Und wieder zwei Fehler im Messgerät

| Fehlgriff | Folge |
|---|---|
| Paketnamen nicht normalisiert | 40 der ersten 53 Meldungen waren Scheintreffer: `cannot use 'pygame.rect.Rect' as a dict key` gegen `'rustgame_py.rect.Rect'` |
| Text auf 70 Zeichen gekürzt **vor** dem Ersetzen | `pygame` sind sechs Zeichen, `@` ist eines — die zwei Seiten brachen an verschiedenen Stellen ab, und 13 gleiche Sätze sahen verschieden aus |

*Wer kürzt, kürzt zuletzt.* Es ist der achte Fehlgriff in einem eigenen
Messgerät in dieser Reihe — und wie jeder davor hätte er mich eine
Behauptung gekostet, wenn ich die Zahl nicht zweimal angesehen hätte.

## Runde zehn: `class Spieler(pygame.sprite.Sprite)`

Die häufigste Zeile in jedem pygame-Spiel ist eine **Ableitung**, und
`class Bild(pygame.Surface)` steht in pygames eigener Doku. In PyO3 ist
das ein ausdrücklicher Schalter:

```rust
#[pyclass(name = "Py_buffer", module = "rustgame_py.newbuffer", unsendable)]
//                                                      ^ kein `subclass`
```

Fehlt er, gibt eine Ableitung `TypeError: … is not an acceptable base
type`. **Neunzehn Runden lang hat keine Probe danach gefragt** — sie
haben gefragt, ob eine Klasse *da* ist, wie sie *heißt*, was sie bei
falschem Aufruf *sagt*, aber nie, ob man von ihr *erben* kann.

179 Messungen über alle 65 Klassen: ableiten, die Unterklasse bauen,
`isinstance`, ein eigenes Feld, ein neu angehängtes. **Drei Klassen
ließen sich hier nicht ableiten**, bei pygame schon:

| Klasse | bei pygame |
|---|---|
| `newbuffer.Py_buffer` | ein C-Typ **mit** `BASETYPE` |
| `sprite.collide_rect_ratio` | eine gewöhnliche Python-Klasse mit `__call__` |
| `sprite.collide_circle_ratio` | dieselbe Bauart |

Drei Wörter `subclass`, und alle drei stimmen.

### Ein pygame-Fehler, der das Messen selbst vergiftet

Beim ersten Lauf brach das Messgerät nach siebzehn Klassen ab — mit einem
Fehler, der nicht dort entstand, wo er auftrat:

```text
TypeError: a sequence is expected
SystemError: <built-in method write of _io.TextIOWrapper …>
             returned a result with an exception set
```

Ein `print` scheiterte. Die Ursache lag eine Zeile vorher:

```python
U = type('U', (pygame.Vector3,), {})
setattr(U(1, 2, 3), 'zusatz', 1)
#   SystemError: setattr returned a result with an exception set
```

**`setattr` auf einer Unterklasse von `pygame.Vector3` lässt eine
Ausnahme im Interpreter stehen.** Der Setzer liest die Zuweisung als
Folge (für `v.xy = (1, 2)`), scheitert still und meldet trotzdem Erfolg —
und der *nächste* beliebige Aufruf erbt den Fehler. `Vector2` hat ihn
nicht; hier geht beides sauber.

Das ist der **dritte** gemessene Fehler in pygame-ce 2.5.7, nach dem
falschen Drehsinn in `rotate_y/z_ip_rad` und dem Absturz bei
`newbuffer.Py_buffer(1)`. Die Probe nimmt diese eine Messung aus — *ein
Messgerät, das das Messobjekt vergiftet, misst nichts mehr, und zwar
auch nichts danach.*

### In die andere Richtung: vier Klassen, die hier mehr erlauben

`_camera.Camera`, `_render.Image`, `_render.Renderer` und
`_render.Texture` sind bei pygame Cython-Typen ohne `BASETYPE`; hier sind
es gewöhnliche Python-Klassen in den eingebetteten Modulen, und von denen
lässt sich immer erben. Das zu verbieten hieße, ein `__init_subclass__`
zu schreiben, das nichts tut als werfen — und es bricht kein Programm:
wer bei pygame nicht ableiten kann, versucht es gar nicht erst.

## Runde elf: `s.dirty = 'x'`

Runde sechs hat jede Funktion mit siebzehn Argumenten gerufen, Runde
sieben mit einem Argument falscher Sorte. Zusammen haben die zwei
**98 echte Fehler** gefunden. Und dann die Frage, die zehn Runden lang
niemand gestellt hat: ein Programm ruft nicht nur, es **weist auch zu**.

```python
sprite.dirty = 2          font.bold = True          circle.radius = 5
font.rotation = 90        color.hsva = (0, 100, 100, 100)
```

1980 Zuweisungen: 31 Instanzen, jedes nicht rufbare öffentliche Feld,
elf Unsinnswerte. **528 gingen auseinander** — mehr als ein Viertel.

| Was | Zuweisungen |
|---|---:|
| **`freetype.Font`s elf Schalter nahmen jeden Wert an.** pygame prüft dort mit `PyBool_Check` und lehnt sogar `1` ab: `f.kerning = 1` gibt `TypeError: The style value must be a boolean`. Hier stand `is_truthy()`, und aus `1.5`, `'x'` und `(1,2,3,4,5)` wurde `True`. | 77 |
| **`font.Font`s vier Schalter lehnten ab, was pygame annimmt.** Dort liest `font.c` `PyObject_IsTrue`: `f.bold = 'x'` ist `True`, `f.bold = []` ist `False`. Hier war es `an: bool`, also wies PyO3 alles außer `True`/`False` zurück. **Genau umgekehrt wie beim Nachbarmodul** — zwei Schriftmodule, zwei Regeln, und hier waren beide verdreht. | 56 |
| **Sprite- und Gruppenfelder waren getypt.** pygames `sprite.py` ist reines Python: `s.dirty = 'x'` geht dort durch, und erst `visible = 1` stolpert mit `TypeError: '<' not supported between instances of 'str' and 'int'` — in pygames eigener Zeile `if self.dirty < 2`. Jetzt liegen `dirty`, `blendmode`, `visible`, `spritedict`, `lostsprites` und `ratio` als Python-Objekte, und der Vergleich geht über Python: **derselbe Satz aus derselben Stelle.** | 75 |
| **`Circle` und `Line` gaben Maschinensätze**, wo pygame vier eigene Sorten hat: `Expected a number, got 'list'` (x, y, r), `Invalid type for area, must be numeric` (die abgeleiteten Maße), `Expected a number` (`Line.ax`) und `Expected a sequence of 2 numbers` (die Punktfelder). | 85 |
| **`underline_adjustment` hatte keine Bereichsprüfung.** `-7`, `1e+30` und `2**31` gingen durch; pygame sagt `underline adjustment value '-7.0' is outside range [-2.0, 2.0]` — mit dem Wert im Satz, als Pythons `repr`. | 3 |
| **`rotation = 10**30` warf hier.** pygame rechnet den Winkel in Pythons Zahlenraum auf `[0, 360)` herunter und gibt **280**. | 2 |
| **`fgcolor` gab `ValueError`, pygame `AttributeError`** — und pygame nimmt dort **nur** Folgen und `Color`: `f.fgcolor = 'red'` ist ein Fehler, `f.fgcolor = b'red'` nicht (Bytes sind eine Folge von Zahlen). | 11 |
| `style` ohne das Hex im Satz (`Invalid style value fffffff9`), `size` mit einem Satz statt pygames vier, `outline`/`align` ohne die Grenze des C-`long` | 219 |

### Der Fund, der ohne Rückmessen unsichtbar geblieben wäre

Eine gelungene Zuweisung ist **kein Beweis**. Die Probe liest den Wert
danach zurück, und genau das fand dies:

```text
c.area = 2**31-1 ; c.area
pygame     2147483646.9999998
rustgame   2147483646.9999995
```

Drei Einheiten im letzten Bit — und sie kamen allein aus einer Klammer:
pygames `circle.c` schreibt `M_PI * circle.r * circle.r`, also
`(PI * r) * r`, und hier stand `PI * (r * r)`. *Wer nur prüft, dass es
keinen Fehler gab, misst die halbe Zuweisung.*

### Die elf Unsinnswerte — und die drei, die die ersten sieben nicht sahen

Der erste Wertesatz hatte sieben Einträge (`'x'`, `None`, `[]`, `-7`,
`10**30`, `(1,2,3,4,5)`, `1.5`) und fand 377 Abweichungen. Dann kamen
`True`, `False` und die beiden Ränder des C-`long` dazu — und mit ihnen
**drei Funde, die kein anderer Wert sehen konnte**:

* `freetype.Font.style = True` wurde hier abgewiesen; pygame nimmt es
  (`bool` ist eine Unterklasse von `int`) und gibt `1`.
* `font.Font.outline = 2**31` wurde hier stillschweigend **0** — der Wert
  ging durch ein `i64` und danach durch ein `as i32`. pygame wirft
  `OverflowError: Python int too large to convert to C long`. **Ein
  stiller falscher Wert ist die schlechteste der drei Antworten.**
* `Circle.x = 2**31` ging hier durch; pygame liest Ganzzahlen über
  `PyLong_AsLong`, und ein C-`long` ist unter Windows 32 Bit. `1e30` als
  **Gleitzahl** nimmt es dagegen an — die Größe ist nicht der Punkt,
  der Typ ist es.

*Ein Wertesatz ist eine Liste, und jede Liste hat blinde Flecken* — die
Lehre der handgepflegten Namenslisten, eine Ebene tiefer.

### Und derselbe Fehler im Messgerät, zum neunten Mal

Der erste Lauf meldete 1115 Zuweisungen bei pygame und 1260 bei rustgame:
147 Felder „nur bei rustgame", alle an `Circle` und `Line`. Die Ursache
war nicht die Hülle, sondern `getattr(pg, "geometry")`: bei pygame ist
das erst ein Feld, **nachdem** jemand `pygame.geometry` selbst eingeführt
hat. Die Probe holt die zwei Klassen jetzt über `importlib`.

### Die zweite Achse derselben Runde: was sagt ein **Wert**?

`alle_untermodule` vergleicht in jedem der sechzig Untermodule die
**Namen**. Die Probe `konstanten` vergleicht **58 Werte** am Hauptmodul —
und die 58 standen dort von Hand aufgeschrieben. Nach allem, was dieses
Projekt über handgepflegte Listen gelernt hat, war das die nächste
Frage.

1898 Werte, abgelesen statt genannt, **mit** ihrem Typ. Drei Funde:

| Fund | pygame | hier |
|---|---|---|
| `freetype.STYLE_DEFAULT` und `_freetype.STYLE_DEFAULT` | `255` | `-1` |
| `sysfont.OpenType_extensions` | `frozenset({'.ttf', '.ttc', '.otf'})` | ein **Tupel** |
| `sysfont.is_init`, `Sysfonts`, `Sysalias` | `True`, 391 Schriften, 33 Zweitnamen | `False`, zweimal leer |

`STYLE_DEFAULT` war **doppelt** falsch: derselbe Wert ist in
`freitext.rs` auch der Platzhalter für „nimm den Stil der Schrift", also
gab `f.style = 255` einen `ValueError` **und** `f.style = -1` setzte
stillschweigend nichts. Keine andere Probe konnte das sehen — der Wert
stand als Rust-Konstante von Hand da und in keiner erzeugten Tabelle.

Die Schriftentabelle war die größere Arbeit: pygames `get_fonts`,
`match_font` und `SysFont` rufen alle drei `initsysfonts()`, und das
füllt `Sysfonts` mit **allen** Schnitten je Name
(`{(False, False): …, (True, False): …}`). Hier stand die Suche im Kern,
und niemand stieß die Python-Seite an. Jetzt sind es auf beiden Seiten
391 Schriften und 33 Zweitnamen, **Eintrag für Eintrag gleich** — und
`is_init` steht, wenn es bei pygame steht.

### Ein `ForwardRef`, der eine Anmerkung unbrauchbar machte

Der dritte Fund dieser Achse sah nach Kosmetik aus und war keine:

```python
def f(c: pygame.typing.ColorLike): ...
typing.get_type_hints(f)
pygame     {'c': pygame.color.Color | … | str | int}
rustgame   NameError: name 'Color' is not defined
```

Die eingebettete `typing.py` schreibt `Union["Color", …]` als
**Vorwärtsverweis**, weil ein `import rustgame_py.color` mitten im
Modulaufbau hängt (das Elternmodul steht noch nicht in `sys.modules`).
Ein `ForwardRef` ist aber kein gleichwertiger Ersatz: wer seine
Anmerkungen zur Laufzeit auflöst, bekam einen `NameError`. Gebraucht
wird dafür **kein Import** — die drei Klassen liegen als Rust-Typen vor,
und das `Union` wird nach dem Modulaufbau mit ihnen neu gebaut. Danach
werden die Hilfsnamen wieder gestrichen, damit `dir(typing)` dieselben
elf Namen zeigt.

## Runde zwölf: jede Methode einmal falsch gerufen

`Fehlertexte` ruft jede freie Funktion mit einem Argument falscher Sorte
— und an **Instanzen** kannte sie sieben Typen: `Vector2`, `Vector3`,
`Rect`, `FRect`, `Color`, `Surface`, `Mask`. Die Methoden von `Font`,
`freetype.Font`, `Sound`, `Channel`, `Circle`, `Line`, den Sprites und
den vier Gruppen hat damit **nie jemand** falsch gerufen.

Dass dort etwas liegt, war aus Runde elf schon bekannt:
`font.set_bold('x')` nimmt pygame an, hier flog ein `TypeError` — und
keine Probe sah es, weil `Font` in keiner Instanzliste stand.

431 Rufe, **46 Abweichungen**:

| Was | Rufe |
|---|---:|
| **`Circle` und `Line`**: pygame hat für fast jede Methode einen eigenen Satz — `move requires a pair of numbers`, `A CircleType object was expected`, `Invalid shape argument, must be a Circle, Rect / Frect or a coordinate` (samt pygames eigenem `Frect`), `project requires a sequence of two numbers`. Hier gab ein gemeinsamer Helfer überall denselben. Und: pygame schreibt an diesen Stellen `two` aus, nicht `2`. | 15 |
| **Die Rekursionswächter — kein Fund**, siehe unten | 16 |
| **`LayeredUpdates.draw('x')` tat hier gar nichts.** pygames erste Zeile ist `surface_blit = surface.blit`, also gibt es `AttributeError: 'str' object has no attribute 'blit'` — auch bei leerer Gruppe. Hier lief der Durchlauf ins Leere, und ein falsches Ziel fiel **nie** auf. | 1 |
| `get_sprites_from_layer('x')` und `remove_sprites_of_layer('x')` geben bei pygame `[]` (es wird nur verglichen, und ein Vergleich mit einer Zeichenkette ist `False`); `get_sprite('x')` gibt Pythons Listensatz; `get_sprites_at('x')` den von `Rect.collidelistall` | 8 |
| `BufferProxy.write('abc')` nimmt pygame an — `s#` liest `str` als UTF-8 mit | 1 |
| `PixelArray.extract('red')`: pygame nimmt dort **nur** `Color` und RGB[A]-Tupel, nicht die Namenstabelle, und sagt das auch (`invalid color (here, generic sequences are restricted, …)`) | 1 |
| `Font.set_linesize('x')`, `Surface.convert_alpha('x')` (**Argument vor Zustand**), `Circle.rotate('x')`, `Line.scale_ip` | 4 |

### Sechzehn Rekursionen, die keine Abweichung sind

`Group.add('methodenmarke')` läuft bei **beiden** in eine
Endlosschleife: eine Zeichenkette ist eine Folge von Zeichenketten, und
`add` steigt in jede Folge hinab. Beide melden `RecursionError`, aber mit
dem Satz ihres eigenen Wächters — `maximum recursion depth exceeded`
gegen `Stack overflow (used 2912 kB) while calling a Python object`.
Unserer trägt sogar eine Zahl, die von Lauf zu Lauf wechselt. Dieselbe
Lage wie bei `<argumentfehler>`: **der Satz stammt von der Laufzeit, nicht
von pygame**, und die Art ist das Einzige, worauf sich ein `except`
verlassen darf.

### Der Fehlgriff dieser Runde: ein `map_err` zu weit

Der erste Anlauf schrieb die neuen Sätze mit einem `map_err` um den
ganzen Aufruf:

```rust
let (f, u) = streckung(args).map_err(|_| PyTypeError::new_err(
    "scale_ip requires a sequence of two numbers"))?;
```

Damit war auch der **richtige** Fehler weg: `line.scale_ip(0, 0.5)` gibt
`ValueError: Can only scale by a positive non zero number`, und daraus
wurde ein `TypeError`. Gemeldet hat es nicht die neue Probe, sondern
pygames eigene Suite — `geometry_test.test_meth_scale_ip`, der einzige
rote Test dieses Zuges. Jetzt bekommt `streckung` den Namen der
gerufenen Methode und schreibt den Satz **dort**, wo er entsteht.

*Ein Fehler, den man gegen einen anderen tauscht, ist kein behobener
Fehler* — die Lehre aus dem `chcp 65001`-Versuch, noch einmal.

## Runde dreizehn: jede Klasse einmal falsch gebaut

`Fehlertexte` lässt Klassen ausdrücklich aus
(`if not callable(w) or isinstance(w, type): continue`), und
`Argumentfehler` baut sie mit **siebzehn** Argumenten. Mit **einem**
falschen hat sie nie jemand gebaut. 105 Klassen, **15 Abweichungen** — und
die erste davon ist die unangenehmste des ganzen Zuges:

```text
pygame     FileNotFoundError: No file 'nixda' found in working directory 'C:\…\rustgame'.
rustgame   FileNotFoundError: Das System kann die angegebene Datei nicht finden. (os error 2)
```

**Ein deutscher Satz in einer Ausnahme** — der Satz von Windows, in der
Systemsprache. `tools/deutsche_fehlertexte.py` meldete weiter **0**, und
zu Recht: der Satz steht nirgends im Quelltext. Der Wächter sucht
deutsche Prosa in Rust-Zeichenketten; dass eine **Fremdmeldung**
durchgereicht wird, war in seinem Begriff von „deutscher Fehlertext" nicht
vorgesehen. Betroffen waren alle sieben Wege zu einer Schrift
(`font.Font`, `font.FontType`, `freetype.Font`, `_freetype.Font`,
`ftfont.Font`, `ftfont.FontType` und `Font` am Hauptmodul).

Jetzt steht dort pygames Satz, samt seiner Unterscheidung:

```text
Font('nixda')          No file 'nixda' found in working directory '…'.
Font('C:/nix/x.ttf')   No such file or directory: 'C:/nix/x.ttf'.
Font('.')              No file '.' found in working directory '…'   (ein Ordner zählt nicht)
Font(b'nixda')         No file 'b'nixda'' found in working directory '…'
```

Die anderen acht: `Mask('x')` gibt bei pygame `size must be two numbers`
statt PyO3s Satz über das Umwandeln eines Tupels (drei Namen), und
`_sdl2.Image('x')` und `_sdl2.Renderer('x')` gaben hier einen
`AttributeError` aus der nächsten Zeile, wo pygame einen `TypeError` mit
dem erwarteten Typ nennt — die **Art** ist das Wichtige, ein
`except TypeError` fing den Fall vorher nicht.

Zwei Fälle bleiben, und sie sind die bekannte Grenze `_render.Image`:
`_render.Image('x')` baut bei pygame anstandslos, weil die Klasse dort
ein **leerer Stummel** ist. Unter `_sdl2.*` stimmen beide wörtlich.

## Runde vierzehn: was `help()` zeigt — und was `inspect` sieht

Die offene Frage aus dreizehn Runden war `__doc__`: pygames C-Funktionen
tragen in der ersten Doku-Zeile ihre Signatur, unsere deutsche Prosa.
Gemessen an **2819** Namen — jedes Modul, jede Klasse, jede Methode, jedes
Feld:

| | pygame | vorher hier |
|---|---:|---:|
| hat einen Docstring | 2501 | 1830 |
| erste Zeile ist eine Signatur | 1384 | 0 |
| **deutsche Prosa** | 0 | 1129 |
| `__text_signature__` | 797 | 1823 |

`help(rustgame_py.draw)` zeigte **nichts**, und `help(pygame.Surface)`
deutsche Sätze über die Umsetzung. Das ist dieselbe Sorte Fehler wie ein
deutscher Fehlertext — nur sieht ihn niemand, der nur Programme laufen
lässt.

### Was geht und was nicht: `__doc__` ist nur halb schreibbar

```python
rustgame_py.draw.__doc__ = "..."          # geht   (Modul)
rustgame_py.Surface.__doc__ = "..."       # geht   (Klasse, auch eine PyO3-Klasse)
rustgame_py.Surface.fill.__doc__ = "..."  # AttributeError: attribute '__doc__' of
                                          # 'method_descriptor' objects is not writable
```

Damit ist die Entscheidung gemessen und nicht gewählt: **Module und
Klassen bekommen pygames Satz, Methoden können ihn nicht bekommen.** Die
Sätze kommen aus `tools/gen_moduldoku.py` — aus **echtem** pygame erzeugt,
48 Module und 94 Klassen, eingecheckt wie `gen_konstanten.py` und
`gen_golden.py`. Der deutsche Vermerk bleibt im Quelltext, wo er
hingehört; pygames Satz wird beim Modulaufbau darübergelegt.

Drei Dinge entstehen **später** als dieser Aufbau und brauchten je eine
eigene Zeile: die faulen Module (`surfarray`, `sndarray`), `ftfont.Font`
und `_camera.Camera`. Beim ersten Versuch stand der Haken im
Stellvertreter `Spaet` — der fängt aber nur `pygame.surfarray.foo`, nicht
`import pygame.surfarray`. Jetzt sitzt er im **Lader**, durch den beide
Wege gehen.

### Und ein Hänger, den dieses Projekt schon kannte

Der erste Entwurf setzte die Klassensätze mit `getattr(modul, name)` —
und blieb hängen. Die Spur (nur mit
`faulthandler.dump_traceback_later` zu sehen) endete in `_camera.py`
Zeile 55: ein Modul-`__getattr__`, das nachlädt, und ein Import
**während** des Modulaufbaus wartet auf sich selbst, weil `rustgame_py`
noch nicht in `sys.modules` steht. Gefragt wird jetzt das `__dict__`.

### Die zweite Hälfte der Runde: 104 Methoden nahmen ein Schlüsselwort zuviel

Auf dem Weg zu `__doc__` fiel `inspect.signature` auf, und damit die
eigentliche Beute:

```text
inspect.signature(pygame.font.Font.set_bold)    (self, object, /)
inspect.signature(rustgame.font.Font.set_bold)  (self, /, an)
```

pygames Methode nimmt **kein** Schlüsselwort (`Font.set_bold() takes no
keyword arguments`), unsere nahm `an=True` — und `an` ist dazu deutsch.
Das ist Runde sechs eine Ebene tiefer: dort waren es 58
**Modulfunktionen**, hier **104 Methoden**, und die Probe von damals
sagte selbst, warum sie sie nicht sah:

> *Nur Modulfunktionen und Klassen, keine Methoden: der Weg einer Methode
> hängt an ihrer Klasse, und `inspect` kommt an die Methodennamen der
> C-Typen nicht heran.*

Das stimmt für die **Namen** und nicht für die **Form**. Ein `/` am Ende
der PyO3-Signatur stellt beides gleich und macht die deutschen Namen
nebenbei unnennbar — `an`, `wert`, `liste`, `groesse`, `tafel`,
`vorlage`, `erstes`, `grad`, `o`.

Dieselbe Messung fand fünf Formfehler, die kein Aufruf zeigt:

| Stelle | pygame | vorher hier |
|---|---|---|
| `event.set_blocked`, `set_allowed`, `get_blocked` | ein **Pflicht**argument (`takes exactly one argument (0 given)`) | `=None`, also weglassbar — `set_blocked()` sperrte hier **nichts** |
| `Window.flash`, `set_icon`, `set_modal_for` | nur nach Stellung | nahmen ein Schlüsselwort |
| `ftfont.Font.render` | vier Parameter | **fünf** — `wraplength` hat pygames `ftfont` nicht (`pygame.font.Font.render` hat es) |
| `_sdl2.Image.get_rect` | `(self)` | `**kwargs` — die nimmt nur `Texture.get_rect`, so steht es in `video.pyi` |
| `joystick.JoystickType.set_led` | ein Argument, nur Stellung | `*args`, mit dem ungemessenen Vermerk „nimmt eine Farbe oder drei Zahlen" |

Die drei Ereignisfunktionen sind der schlimmste Fall davon: ein
`set_blocked()` ohne Argument sah hier wie Erfolg aus und sperrte nichts.

### Der Fehlgriff dieser Runde: ein Wächter, der selbst zum Wert wird

Jede Probe hat eine Untergrenze (`if len(aus) < N: raise`), damit eine
Probe ohne Einträge nicht als „gleich" durchgeht. Und `probe()` macht aus
einer Ausnahme eine Zeichenkette, damit eine Probe, die bei **einer**
Engine wirft, auffällt.

Zusammen ergibt das eine Lücke, in die ich in dieser Runde **zweimal**
getreten bin: steht die Untergrenze zu **hoch**, wirft die Probe bei
beiden Engines, die zwei Zeichenketten sind gleich, und der Lauf meldet
„0 Abweichungen", ohne einen einzigen Wert verglichen zu haben. Die
Probe `Dokumentation` hat so zwei Läufe lang nichts gemessen (Grenze 200,
dann 150, bei 142 echten Namen).

Der Riegel sitzt jetzt im Lauf selbst, nicht in der Hoffnung: `main()`
sucht nach Proben, deren Wert mit `RuntimeError: ` beginnt, nennt sie und
endet mit 1. Nachgeprüft, indem eine Grenze absichtlich auf 999 gesetzt
wurde — ein Messgerät, das man nicht geprüft hat, ist keins.

## Runde fünfzehn: `pygame.quit()`, und dann noch was

Vierzehn Runden haben gefragt, was ein Name tut, **wenn alles läuft**.
Ein Programm macht aber das:

```python
pygame.quit()
...
pygame.event.get()        # und jetzt?
```

Und vor dem ersten `pygame.init()` genauso. Drei Zustände, 125 Fragen,
jeder Zustand in einem **eigenen Prozess** — ein `quit()` lässt sich
nicht zurückdrehen:

| Zustand | Abweichungen |
|---|---:|
| `vorher` — frisch eingeführt | **35** |
| `laeuft` — `init()` und `set_mode` | 1 |
| `nachher` — nach `quit()` | **34** |

**80 von 375**, und fast alle in derselben Richtung: *pygame sperrt, wir
antworten.* Ein Programm, das `init()` vergisst, bekam hier `(0, 0)` für
die Mausposition und `[]` für die Ereignisse — die schlimmste Sorte
Unterschied, weil nichts auffällt.

### Die Tortabelle, gemessen statt hergeleitet

Welche Funktion ein Tor hat, ist von außen nicht zu erraten. Also wurde
pygame in **sechs** Zuständen gefragt (nichts, `display.init()`,
`font.init()`, `freetype.init()`, `mixer.init()`, `pygame.init()`), und
heraus kam eine klare Tabelle:

| Tor | sperrt |
|---|---|
| **Videoteilsystem** (`video system not initialized`) | `event.*` (11), `key.*` (8), `mouse.*` (9), `display.Info`, `list_modes`, `flip`, `update`, `toggle_fullscreen`, `get_driver`, `mode_ok`, `get_desktop_sizes`, `get_allow_screensaver`, `Surface.convert` |
| **`font.init()`** (`font not initialized`) | nur `font.Font` |
| **`freetype.init()`** (`RuntimeError: The FreeType 2 library hasn't been initialized`) | `freetype.Font`, `get_error` |
| **`mixer.init()`** (`mixer not initialized`) | 16 Stellen, darunter `music.get_pos` |
| **SDLs Zeitgeber** (`pygame is not initialized`) | nur `time.set_timer` |

Und was **kein** Tor hat, ist genauso wenig herzuleiten: `key.name`,
`key.key_code`, `event.custom_type`, `mouse.get_relative_mode`,
`time.delay`, `time.wait`, `font.get_fonts`, `font.match_font`,
`freetype.get_version` — und `display.get_num_displays` gibt ohne Video
schlicht `0`, `display.get_active` `False`.

### Vier Funde, die keine Tore sind

1. **`pygame.get_init()` war immer `True`.** Es fragte
   `rustgame_core::time::get_init()`, und die Zeitkiste fährt beim ersten
   Zugriff von selbst hoch. pygame sagt `False`, solange `pygame.init()`
   nicht lief — und auch nach `display.init()` allein. Jetzt ein eigener
   Schalter.
2. **`display.get_caption()` gab immer `('pygame window', 'pygame
   window')`.** pygame gibt ein **leeres Tupel**, solange kein Fenster
   offen ist und niemand einen Titel gesetzt hat. Im Quelltext stand das
   Gegenteil, mit dem Vermerk „gemessen" — wahr für den Zustand *mit*
   Fenster.
3. **`font` und `freetype` teilten einen Schalter.** Bei pygame sind es
   zwei: `font.init()` macht `freetype.get_init()` **nicht** wahr.
   Gemessen in vier Kombinationen.
4. **`freetype.get_cache_size()` meldete 64 ohne `init()`**, pygame `0`
   (und nach `init(cache_size=100)` eben 100).

### Und einer, der zwölfmal dastand

```text
pygame     AttributeError: module 'pygame.camera' has no attribute 'get_init'
rustgame   AttributeError: get_init
```

pygames Untermodule haben gar kein `__getattr__` — der Satz kommt von
CPython. Unsere eingebetteten Module werfen `AttributeError(name)`, und
damit ist der Satz der bloße Name. **Zwölf Stellen**, alle gleich, alle
jetzt mit Pythons Wortlaut.

### Der Fehlgriff dieser Runde: der Riegel saß eine Ebene zu hoch

`font.SysFont` bekam den Schriftriegel — und `pygame.ftfont.SysFont`
benutzt **dieselbe Suche** mit einem eigenen Bauer, der an freetype
hängt. `ftfont_test.test_SysFont` wurde rot mit `font not initialized`,
obwohl der Test `ftfont.init()` gerufen hatte. pygame hat den Riegel
dort, wo er hingehört: **im Erzeuger**, nicht in der Suche. Gemeldet hat
es wieder pygames eigene Suite, nicht die neue Messung.

Dazu dreimal das Messgerät, zum elften bis dreizehnten Mal in fünfzehn
Runden: `pg.camera` ohne Import gefragt (pygame hat den Namen dann nicht
— dieselbe Falle wie `geometry` in Runde 11), `str(...)[:8]` in der
**Frage** statt in der Ausgabe (damit griff die Namensnormalisierung
nicht mehr), und `get_error` als Frage, deren Antwort die Fragen davor
sind.

### Ein neues ständiges Werkzeug

Die Messung braucht drei Prozesse, also kann sie keine Probe in
`vergleich_module.py` sein. Sie steht als **`tools/zustand.py`** daneben
— dieselbe Bauart wie `uebersprungen.py`, das aus demselben Grund beide
Engines selbst startet. 360 Fragen, und der Lauf endet mit 1, sobald eine
Antwort auseinandergeht.

## Runde sechzehn: `memoryview(surface.get_buffer())`

Fünfzehn Runden haben den Puffer **benutzt**: `length` abgefragt, Bytes
gelesen, Pixel zurückgeschrieben. Keine hat ihn **beschrieben**. Das
Pufferprotokoll meldet aber elf Kennzahlen, und ein Programm, das numpy
oder `memoryview` benutzt, liest genau die:

```python
m = memoryview(s.get_view("2"))
m.format, m.itemsize, m.ndim, m.shape, m.strides, m.suboffsets
m.readonly, m.nbytes, m.c_contiguous, m.f_contiguous, m.contiguous
```

Also alle elf, für **jeden** Ausgeber: `Surface` in fünf Tiefen mal
vierzehn Zugriffen (`get_buffer`, zwölf Arten von `get_view` und die
Fläche selbst), `Mask` in drei Breiten, `Color`, `PixelArray`, eine
Scheibe davon und `Sound` — **77 Ausgeber, 722 Kennzahlen**. Erster
Lauf: **56 abweichende Zeilen.**

### `.raw` gab Bytes heraus, die so nirgends im Speicher stehen

Dreizehn der 56 kamen von einem einzigen Versäumnis. pygames
`bufferproxy_get_raw` fragt `pgBuffer_IsContiguous(.., 'A')` und gibt
sonst `ValueError: the bytes are not contiguous`. Das `A` heißt **C oder
Fortran**, und das ist nicht `memoryview.c_contiguous`:

| Sicht | `shape` | `strides` | zusammenhängend | `.raw` |
|---|---|---|---|---|
| `get_view('0')` | `(48,)` | `(1,)` | C | ok |
| `get_view('1')` | `(12,)` | `(4,)` | C | ok |
| `get_view('2')` | `(4, 3)` | `(4, 16)` | **Fortran** | ok |
| `get_view('3')` | `(4, 3, 3)` | `(4, 16, -1)` | keines | `ValueError` |
| `get_view('r')` | `(4, 3)` | `(4, 16)` | keines | `ValueError` |

Die beiden letzten Zeilen sind der Witz: `'2'` und `'r'` haben
**dieselben** Schrittweiten. Bei `'2'` ist `itemsize` 4, also sind
`(4, 16)` genau die Fortran-Schritte; bei `'r'` ist `itemsize` 1, und
dann sind dieselben Zahlen keine. Hier gab `.raw` für alle fünf Bytes
heraus — für die Kanalsichten sogar **zusammengesuchte**, die im
Speicher so nirgends liegen. Wer sie an eine C-Bibliothek weitergibt,
bekommt kein Bild, sondern Streifen.

### Die Maske sagt, wie breit ein `unsigned long` ist

`Mask` meldete `I`, pygame `L`. Beides sind vier Bytes — unter Windows.
pygames Maske legt `unsigned long` ab, und das sind unter Linux acht;
unsere ist immer `u32`. Also ist `L` dort, wo es stimmt, der **gleiche**
Typ und sonst eine falsche Zusage: `cfg!(windows)` entscheidet, und der
Unterschied in `itemsize` auf einem 64-Bit-Linux bleibt als das stehen,
was er ist — ein Unterschied in der Maske, nicht im Namen.

### `strides: None` ist kein Weglassen

numpys Abrede sagt, `strides: None` heiße C-Reihenfolge. pygame nimmt
den Schlüssel so aber **nicht** an:

```text
ohne den Schlüssel      length = 4
strides = None          ValueError: expected a tuple of ints for 'strides'
strides = (1,)          length = 4
```

Im Quelltext stand `Some(o) if !o.is_none()`, und damit war `None`
dasselbe wie „nicht da". Ein Programm, das sein Wörterbuch aus numpys
`__array_interface__` kopiert, trifft das sofort — dort steht `strides`
regelmäßig auf `None`.

### Der Erzeuger hat etwas gelesen, was er nicht lesen durfte

Nach diesen drei Funden blieben sieben Zeilen, und die waren eine
Reihenfolge. `BufferProxy({...})` liest bei pygame beim Anlegen
**nichts**; alle sieben Einwände kommen beim ersten Gebrauch:

```text
BufferProxy({})              Erzeuger: ok
.length                      ValueError: required 'shape' item is missing
```

Für ein fremdes Objekt war das hier schon so, mit Begründung und
Messung — für das Wörterbuch nicht. Und es liest so lange neu, bis es
**einmal** gelingt; gemessen:

```text
d = {}; b = BufferProxy(d)
b.length                     ValueError (shape fehlt)
d.update(gültige Angaben)
b.length                     4        ← jetzt gelesen
d["shape"] = (8,)
b.length                     4        ← gemerkt, nicht neu gelesen
```

Damit war der Fall `Quelle::Fest` — eine beim Anlegen fertig gelesene
Beschreibung — tot; das Wörterbuch war der letzte, der ihn gebaut hat.

### `before` feuerte für einen Puffer, den es nie gab

Dieselbe Messung mit einem mitschreibenden `before`-Rückruf trennte eine
Reihenfolge, die man sonst für eine hält:

| Zugriff | pygame | hier (vorher) |
|---|---|---|
| `.length` auf ein gutes Wörterbuch | `before` 1× | `before` 1× |
| `.length` auf ein kaputtes | **`before` 0×**, Fehler | `before` 1×, Fehler |
| `.parent` | `before` 1× | **0×** |
| `repr(bp)` | `before` 1× | **0×** |

pygames `_get_buffer` holt zuerst den Puffer und ruft `before` **nur,
wenn das gelungen ist**. Hier lief es umgekehrt. Und `.parent` holt ihn
überhaupt — es gibt `view.obj` heraus, nicht ein mitgeführtes Feld:
bei einem kaputten Wörterbuch also einen Fehler statt eines `None`, und
ein fremder Ausgeber wird dabei wirklich gefragt. Beides steht jetzt in
**einem** Helfer, `geholt()`, und jeder der sieben Zugriffe geht
darüber.

### Der Fehlgriff dieser Runde: eine Begründung, die nur zufällig stimmte

An `__array_interface__` stand ein Durchreichen der gegebenen Angaben,
mit einem Vermerk, der einen pygame-Test nannte: er lege `data=(0, True)`
hinein und vergleiche das Ergebnis damit. Der Test (`view_keywords`) gibt
alle vier Schlüssel mit — und für **die** ist Durchreichen und Rechnen
dasselbe. Sobald einer fehlt, nicht mehr:

```text
gegeben                       pygame gibt
shape (3,2) typestr <u4       strides (8, 4)      ← gerechnet
shape (4,)  typestr |V3       typestr <V3         ← umbenannt
shape [4]   (eine Liste)      ValueError expected a tuple of ints
```

`strides` fehlte hier also immer, wenn niemand sie hereingab. Dass das
Rechnen verlustfrei ist, musste gemessen werden, bevor der Vermerk
fallen durfte: dreizehn Formate hin und zurück, `|u1 |i1 <u2 >u2 <i2
<u4 >u4 <i4 <f4 <f8` kommen Zeichen für Zeichen wieder, `|V3` wird auch
bei pygame zu `<V3`, und `|b1` lehnen beide ab. *Ein Vermerk, der einen
Test nennt, belegt nur die Fälle, die der Test hat.*

### Zwei neue ständige Proben

* **`Pufferprotokoll`** — 722 Kennzahlen, 77 Ausgeber. Untergrenze 700,
  damit eine Probe, die ihre Einträge verliert, nicht als „gleich"
  durchgeht.
* **`Puffer spaet lesen`** — 85 Antworten: acht kaputte Wörterbücher mal
  sieben Zugriffe, das Nachfüllen, und `before` für neun Zugriffe mal gut
  und kaputt.

## Runde siebzehn: tausend Ereignisse, und ein Objekt ohne `__init__`

Sechzehn Runden haben die Ereignisschlange mit **drei** Ereignissen
gefragt. Ein Spiel stellt pro Bild dreißig hinein und holt sie
stapelweise:

```python
for e in pygame.event.get([pygame.QUIT, pygame.KEYDOWN]):
    ...
```

Gefüllt gefragt — 1 bis 20.000 Ereignisse, jede Form von `get`, `peek`,
`clear`, `set_blocked` — kamen **sieben** Funde heraus, und der erste
betrifft genau die Zeile oben.

### Die Reihenfolge hängt an der Liste, nicht an der Schlange

Gemessen an einer Schlange `A B C D A B C D A B C D`:

| Aufruf | pygame gibt | hier (vorher) |
|---|---|---|
| `get([A, C])` | `A A A C C C` | `A C A C A C` |
| `get([C, A])` | `C C C A A A` | `A C A C A C` |
| `get([D, B, A])` | `D D D B B B A A A` | `A B D A B D A B D` |
| `get(exclude=[A, B])` | Rest `A A A B B B` | Rest `A B A B A B` |
| `get(exclude=[B, A])` | Rest `B B B A A A` | dasselbe |

Die zweite Zeile ist der Beweis: **die Liste entscheidet.** pygames
`pg_event_get` läuft sie durch und fragt für jeden Typ einzeln bei SDL
nach; hier lief ein Durchgang mit einer Mengenprüfung, und das gab
Schlangenfolge. Wer `get([QUIT, KEYDOWN])` schreibt, um das Beenden
*vor* den Tasten zu sehen, bekam die Tasten zuerst.

Doppelte Einträge zählen einmal (`get([A, A, C])` gibt `A A A C C C`),
und ein Tupel verhält sich wie eine Liste — auch das gemessen.

### `set_blocked` räumt die Schlange aus

```text
30 Ereignisse, je zehn A B C
set_blocked(KEYDOWN)   danach 20   ← die zehn sind weg
set_blocked(None)      danach 0
set_blocked([])        danach 30
set_allowed(…)         danach 30, in jeder Form
```

Das kommt von SDL: `SDL_EventState(typ, SDL_IGNORE)` ruft
`SDL_FlushEvent(typ)` mit. Hier blieben die gesperrten Ereignisse liegen
— ein Programm, das einen Typ sperrt, bekam ihn danach trotzdem noch
einmal.

### „Alles sperren" sind nicht alle 65535 Zahlen

```text
set_blocked(None); dann get_blocked für 0..65534
pygame   32818 Typen gesperrt
hier     65535
```

Und die 32818 sind keine Liste, sondern eine Regel: **jeder Typ mit
Namen**, dazu die zwei Gesten `DOLLARGESTURE` und `DOLLARRECORD` (2048
und 2049 — die kennt pygame, benennt sie aber nicht) und der ganze
SDL-Nutzerbereich ab `0x8000`. Nachgerechnet: von den 32818 haben 32742
einen Namen, die 76 anderen sind die zwei Gesten und 32791…32864.

Wer sich einen eigenen Typ unter 256 ausdenkt, kommt bei pygame nach
einem `set_blocked(None)` weiterhin durch:

```text
set_blocked(None); post(Event(1))     True    get_blocked(1)   False
set_blocked(None); post(Event(256))   False   get_blocked(256) True
```

### Drei Fehlerarten, und sie hängen an einem C-`int`

Was als Ereignistyp gilt, ist nicht herzuleiten. Also Wert für Wert
gefragt:

| Wert | pygame |
|---|---|
| `0` … `65534` | gut |
| `65535`, `-1`, `2**31-1` | `ValueError: event type out of range` |
| `2**31`, `2**63` | `TypeError: type sequence must contain valid event types` |
| `"x"`, `b"0"`, `range(3)` | als **Folge** gelesen (ein Text scheitert an seinen Zeichen) |
| `1.5` | `TypeError: event type must be numeric or a sequence` |
| `{0: 1}`, `{0, 1}` | dasselbe — keine Folgen für C |
| `True` | **gut**, ein `bool` ist eine Zahl |
| `[1.9]`, `[-0.5]` | gut, wird `1` bzw. `0` — abgeschnitten, nicht gerundet |

Die Grenze zwischen `ValueError` und `TypeError` ist die Breite eines
C-`int`: `2**31-1` passt hinein und ist „außerhalb des Bereichs",
`2**31` passt nicht und ist „kein Ereignistyp". Hier stand dafür eine
einzige Prüfung, und drei Dinge waren falsch: ein `bool` wurde
**abgewiesen** (mit dem Vermerk „ein `bool` ist in Python eine Zahl, aber
kein Ereignistyp" — gemessen ist er einer), eine zu große Zahl gab
`ValueError`, und alles, was weder Zahl noch Folge ist, bekam den
Folgensatz statt des eigenen. Dazu `get_blocked(None)`: überall sonst
heißt `None` „alle Typen", dort ist es ein Fehler — hier kam `False`
heraus, also eine Antwort auf eine Frage, die pygame nicht beantwortet.

### Eine Liste wird **stückweise** angewandt

```text
get([A, kaputt])          TypeError — und die A sind weg
get([kaputt, A])          TypeError — nichts angefasst
clear([A, kaputt])        TypeError — die A sind geleert
set_blocked([B, kaputt])  TypeError — B ist gesperrt und ausgeräumt
peek([A, kaputt])         **True**, kein Fehler
peek([kaputt, A])         TypeError
get_blocked([0, kaputt])  **True**, wenn 0 gesperrt ist
```

pygame liest jedes Glied und *tut es sofort*; der Fehler hält die
Schleife dort an, wo er auftritt. Und `peek` wie `get_blocked` hören beim
ersten Treffer auf — darum sehen sie das kaputte Glied dahinter nie. Das
ist nachgebaut, bis auf einen Fall: siehe die Grenze
`Typliste halb angewandt`.

### Vier Sätze, die niemand gelesen hatte

* `post()` ohne Argument: `pygame.event.post() takes exactly one argument
  (0 given)` — mit dem ganzen Weg davor und der Zahl dahinter.
* `post(x=1)`: `pygame.event.post() takes no keyword arguments`.
* `custom_type()` am Anschlag (nach **32669** Aufrufen):
  `pygame.event.custom_type made too many event types.`
* `Event(None)`: `'NoneType' object cannot be interpreted as an integer`
  — und hier stand davor `argument 'r#type': `, mit dem **Rust-Rohnamen**
  samt Raute in einer Python-Meldung.

### Die zweite Hälfte: ein Objekt, das `__new__` hatte und kein `__init__`

Aufgefallen ist sie beim Messen der Schlüsselwortsätze: pygame trennt
`tp_new` (legt an) und `tp_init` (liest Argumente), und zwischen beiden
gibt es einen Zustand, den ein Programm erreichen kann — `copy`, `pickle`
und pygames eigener `freetype_test.nullfont()` gehen genau dort hin.

**pygame stürzt dabei ab.** Der erste Lauf fragte alle Klassen in einem
Prozess und endete bei `font.Font.italic` mit
`Fatal Python error: pygame_parachute: (pygame parachute) Segmentation
Fault`; 341 der 447 Fragen waren danach nicht mehr gestellt. Also je
Klasse ein eigener Prozess — und der Absturz wird zur Antwort.

**374 von 507** Antworten gingen auseinander. Die größte Gruppe ist
`freetype.Font`, und dort stand im Quelltext ein Vermerk, der in beiden
Hälften falsch war: „jede Auskunft gibt `RuntimeError: Font not
initialized`".

| Zugriff an der Nullschrift | pygame |
|---|---|
| `antialiased`, `use_bitmap_strikes` | `True` |
| `kerning`, `oblique`, `origin`, `pad`, `strong`, `ucs4`, `underline`, `vertical`, `wide` | `False` |
| `size` `0.0`, `style` `0`, `resolution` `0`, `rotation` `0` | |
| `strength` | `0.027777777777777776` (also `1/36`) |
| `fgcolor` `Color(0,0,0,255)`, `bgcolor` `Color(0,0,0,0)` | |
| `ascender`, `descender`, `height`, `scalable`, `fixed_width`, `fixed_sizes` | `RuntimeError: _freetype.Font instance is not initialized` |
| `name`, `style_name` | `AttributeError: <uninitialized Font object>` |
| `path` | `AttributeError: path unavailable` |
| `repr` | `<uninitialized Font object at 0x…>` |

Die Trennlinie ist **die Datei**: was im eigenen Zettel steht, wird
beantwortet; was die Schriftdatei braucht, nicht. pygames eigener Test
prüft genau die sechs Zugriffe aus der vorletzten Zeile — die anderen
dreiundzwanzig standen hier auf `RuntimeError`, ohne dass es auffiel. Und
der Satz war der von `pygame.font`, nicht der von `pygame.freetype`; nach
einem `freetype.quit()` gilt dort `Invalid freetype font (freetype module
quit since freetype font created)`, dreimal `freetype` im Satz.

### Zwei `__repr__`, und beide standen falsch

```text
pygame.font.Font(None, 13)         pygame   <pygame.font.Font object at 0x…>
                                   hier     <Font(size=8)>
pygame.freetype.Font(None, 13)     pygame   Font('…\pygame\freesansbold.ttf')
                                   hier     <… object at 0x…>
```

Zwei Klassen mit demselben Namen: die eine hatte hier einen `__repr__`
**zuviel**, die andere einen zu wenig. Dazu gab `freetype.Font.path` den
bloßen Dateinamen (`freesansbold.ttf`) statt des ganzen Pfades —
gesucht wird die Datei ohnehin, gemeldet wurde nur nicht, was gefunden
wurde.

### Eine Grenze, die gefallen ist: `Color()`

`Color()` stand als benannte Grenze: PyO3 kenne nur `#[new]`, also seien
`Color()` und `Color.__new__(Color)` nicht zu unterscheiden — der Vorzug
ging an `__new__`, und `Color()` gab eine schwarze Farbe statt des
`ValueError`. Der Ausweg stand seit Runde dreizehn eine Datei weiter:
eine PyO3-Klasse ist ein **Heap-Typ**, und
`setattr(Klasse, "__init__", f)` setzt dort `tp_init` mit. Dasselbe gilt
jetzt für `geometry.Circle` und `geometry.Line`, deren `__new__` hier den
Erzeugerfehler warf, wo pygame `Circle((0.0, 0.0), 1.0)` gibt.

Übrig bleiben `Event` und `Channel`: dort müsste ein **erfundener**
Leerzustand in den Kern (ein Ereignis ohne Wörterbuch, eine Spur ohne
Mischer). Das steht als benannte Grenze mit Messung.

### Der Fehlgriff dieser Runde: eine Änderung, die eine Methode mitnahm

`Line.__new__` gibt jetzt ohne Argumente den Nullzustand — und
`Line.update()` ging denselben Weg. Damit tat ein leeres `update()`
stillschweigend nichts, wo pygame einen `TypeError` gibt. Gefunden hat
das **pygames eigener Test** (`geometry_test.test_meth_update`), eine
Stunde nach der Änderung: der einzige rote von 2364. `Circle.update`
hatte die Prüfung längst, `Line.update` nicht. *Wer einen Erzeuger
großzügiger macht, muss jeden nachsehen, der ihn benutzt.*

### Fünf Fehler im Messgerät, in einer Runde

Die bisherige Bestmarke waren drei. Der Reihe nach:

1. **Die Modulliste kam aus `__all__` jeder Engine.** Unsere hat sechs
   Module mehr (`base`, `imageext`, `macosx`, `newbuffer`, `pkgdata`,
   `pypm` — eine benannte Grenze), also wurden **51 Namen nur auf einer
   Seite** gefragt und 1871 gegen 1922 Antworten verglichen. Die Frage
   kommt jetzt aus pygames Paket.
2. **Eine Untergrenze über dem echten Wert.** Die neue Probe zählte 32
   Antworten, die Grenze stand auf 40 — beide Engines warfen, und der
   Wächter aus Runde vierzehn meldete sofort „diese Proben haben nichts
   gemessen". Zum vierten Mal dieselbe Falle, zum ersten Mal von der
   Mechanik gefangen.
3. **`pg.geometry` ohne `importlib`** — dieselbe Falle wie in Runde elf
   und fünfzehn; pygame hat den Namen dann nicht, und drei „Funde" waren
   `AttributeError`.
4. **`str.replace("pygame", "@")`** traf das Wort im **Pfad**
   (`…/pygamez/.venv/…`), und ein Fenstertitel sah dadurch verschieden
   aus, der gleich war.
5. **Alle Klassen in einem Prozess** — siehe oben: pygames Absturz nahm
   341 Fragen mit.

Dazu ein Fehler, den die **eigene neue Probe** eine Stunde nach ihrer
Geburt fand: im Zweig für eine einzelne Zahl verwarf `typen_je` die
Antwort des Rückrufs, und damit gab `get_blocked(1)` immer `False`.

### Zwei neue ständige Proben und ein neues Werkzeug

* **`Ereignisschlange unter Last`** — 32 Antworten: Fassungsvermögen,
  Reihenfolge in sechzehn Formen, Sperren und Freigeben, 300 Ereignisse
  mit Nutzlast durch `poll`.
* **`Ereignistypen als Argument`** — 192 Antworten: 32 Werte × 6 Wege,
  und nach jedem die Frage, **was** nun wirklich gesperrt ist.
* **`Typliste halb angewandt`** — zehn Antworten, davon zwei als
  benannte Grenze.
* **`tools/halbfertig.py`** — 463 Fragen an 51 Klassen, je Klasse ein
  eigener Prozess. Keine Probe, weil pygame abstürzt.

## Runde achtzehn: PyGameZ selbst

Siebzehn Runden haben gefragt, ob rustgame sich verhält wie pygame —
Funktion für Funktion, Feld für Feld, Zustand für Zustand. Keine hat das
getan, wofür es gebaut ist: **PyGameZ laufen lassen.** Im Plan stand es als
Endabnahme („PyGameZ zweimal starten — einmal mit echtem pygame, einmal
mit rustgame — und die Bilder vergleichen"), gefahren wurde
`vergleich_spiel.py`, ein Testspiel von 180 Zeilen, das den Text
ausdrücklich aus dem Bild hält.

Zwei neue ständige Werkzeuge stellen die Frage jetzt an die echte Sammlung:

* **`tools/vergleich_pygamez.py`** — jedes der 46 Spiele aus
  `games.ALL_GAMES`, je Engine ein eigener Prozess, feste Uhr, fester
  Zufall, dasselbe Eingabeskript, 300 Bilder, jedes als SHA-256.
* **`tools/pygamez_audits.py`** — PyGameZ' eigene 14 Audits (11.516
  Zeilen, die niemand für rustgame geschrieben hat) auf beiden Engines.

### Was der erste Lauf zeigte

Mit echtem Text war **jedes** Bild verschieden — das war erwartet (die
benannte Grenze `font.metrics glyphenbox`). Also bekam die Messung eine
Ersatzschrift aus Rechtecken, auf beiden Seiten gleich, damit alles
andere sichtbar wird. Dann blieben drei Dinge:

| Fund | Wirkung | Ursache |
|---|---|---|
| eine Zeile bei y=213, eine Spalte bei x=631, je um 1 daneben | **jedes** Bild jedes Spiels | `smoothscale`: rustgame rechnete nur `GENERIC`, pygame `SSE2` |
| das Lächeln des Minesweeper-Smileys, ein Rand bei Sudoku | 2 Spiele | `draw.arc` nahm die Winkel als `f32`, pygame als `double` |
| alle Zeitbudgets der Audits gerissen | jedes Audit mit Tempo-Prüfung | `blit`, `fill`, `smoothscale` 15- bis 30-mal langsamer als pygame |

### `smoothscale`: eine Grenze, die keine war

In `transform.rs` stand, rustgame rechne `GENERIC`, weil pygames zwei
Rechenwege „nicht einmal miteinander pixelgleich" seien und man an
dieser Stelle „nicht näher herankomme, auch pygame selbst nicht". Die
Messung (98 Fälle, sieben Quell- mal sieben Zielgrößen, mit und ohne
Alphakanal):

| | gegen pygame `GENERIC` | gegen pygame `SSE2` (die Vorgabe) |
|---|---:|---:|
| rustgame vorher | 0 abweichend | **74** abweichend |
| rustgame jetzt, `SSE2` | — | **0** |
| rustgame jetzt, `GENERIC` | **0** | — |

Der Satz verwechselte zwei Fragen: dass pygames zwei Wege *voneinander*
abweichen, sagt nichts darüber, ob man *einen* davon treffen kann. `SSE2`
ist 16-Bit-Ganzzahlarithmetik — 14-Bit-Festkomma, `mulhi` schneidet ab,
die Spuren dürfen überlaufen, `packus` liest mit Vorzeichen —, und jede
davon lässt sich Spur für Spur nachrechnen. Übertragen aus
`simd_transform_sse2.c`; vorgegeben ist jetzt `SSE2`, wie bei pygame.

Und eine Toleranz hatte es verdeckt: die vier `smoothscale`-Referenzbilder
trugen `tol=2` — „pygame ist dort nicht einmal mit sich selbst
pixelgleich". Jetzt `tol=0`, alle vier grün. *Eine Toleranz ist ein
Vermerk, und ein Vermerk ist der bequemste Platz für einen Fehler.*

Nebenbei: `set_smoothscale_backend("MMX")` sagt bei pygame
`Unknown backend type MMX`, nicht „not supported" — der Ersatzzweig
vergleicht den **bisherigen** Weg statt des gewünschten. Hier stand das
Gegenteil; die Probe hatte nur `XYZ`, `NEON` und `GENERIC` gefragt.

### `draw.arc`: das letzte Bit von π

pygame liest die zwei Winkel mit `"O!OOdd|i"` — als `double`. `draw.arc`
ist die **einzige** Funktion in `draw.c`, die das tut; hier stand `f32`.
`math.pi` als `f32` ist 3,1415927, also *größer* als π, und genau an den
Achsen, wo ein Bogen beginnt oder endet, entscheidet das letzte Bit über
das Randpixel: **1887 von 4116** Bögen lagen daneben. Nachgewiesen in
beide Richtungen — pygame mit auf `f32` gerundeten Winkeln traf die alte
Fassung in 4116 von 4116 Fällen. Der Algorithmus war richtig, nur die
Zahl nicht. Ein Fund ist ein Hinweis auf seine Geschwister: alle anderen
Winkel und Faktoren (`rotate`, `rotozoom`, `scale_by`, `Rect.scale_by`)
sind in pygames C wirklich `float` — nachgesehen, nicht angenommen.

### Drei Fehler im Messgerät, bevor es etwas maß

1. **Acht Spiele bauen `random.Random()` ohne Saat** — das zieht aus
   `os.urandom`. Der erste Lauf meldete zehn abweichende Spiele; acht
   davon waren auf **derselben** Engine nicht wiederholbar. Feste Saat je
   Generator, `PYTHONHASHSEED=0`, und `--zweimal` prüft es.
2. **`arcade_casino_audit` startet die anderen Audits als Kindprozesse**
   mit `sys.executable` — die liefen auf echtem pygame, und „rustgame:
   ALLES OK" maß zur Hälfte die Vorlage. Engine und Datenordner gehen
   jetzt über ein `sitecustomize.py` an jeden Prozess.
3. **Die echte Statistik.** Die Audits leiten `store._PATH` um, aber
   `stats` schreibt über `store.path_for` in `store._DIR`. Mein erster
   Lauf hat so **1125 Duell-Einträge** in `stats.pgzlamastats`
   geschrieben (die Datei hatte schon tausende aus früheren Audit-Läufen).
   Jetzt bekommt jeder Lauf einen eigenen leeren Datenordner, und das
   Werkzeug prüft am Ende per SHA-256, dass alle acht Nutzerdateien
   unverändert sind — sonst endet es mit 1.

### Das Ergebnis

**46 von 46 Spielen sind über je 300 Bilder Byte für Byte gleich** —
13.800 Bilder, 12,7 GB verglichene Bytes, gegen pygames Vorgabeweg (mit
Ersatzschrift).

### Geschwindigkeit ist Verhalten

PyGameZ' Audits haben Zeitbudgets — 60 Bilder je Sekunde heißt 16,7 ms.
Der erste Lauf auf rustgame bestand **jede inhaltliche Prüfung** und riss
**17 Budgets in neun Audits**: 2048 bei 1280×960 brauchte 38,8 ms je Bild,
pygame 1,5 ms. `cProfile` sieht auch Aufrufe in C und PyO3, und es zeigte
eine einzige Stelle: **96 % der Zeit in `Surface.blit`** — 10.000 Blits
kosteten 7,9 s, bei pygame 0,25 s.

Die allgemeine Blit-Schleife fragte **je Pixel** das Format beider Puffer
ab (`nach_argb` und `von_argb` vergleichen ganze `Format`-Werte), die
Farbtafel, den Blitweg und die Mischart. Ein Vollbild-Hintergrund hat 1,2
Millionen Pixel. Drei Änderungen, alle ohne ein einziges anderes
Ergebnis:

| Stelle | was | Wirkung |
|---|---|---|
| `blit_32` | zwei 32-Bit-Puffer ohne Farbtafel: jede Weiche **vor** der Schleife, die Mischart als Nulltyp eingesetzt, im häufigsten Fall (kein Schlüssel, kein Flächen-Alpha) eine weichenfreie Schleife je Kombination | `SRCALPHA` 2,93 → 0,86 ms, `BLEND_RGB_ADD` 1,72 → 0,28 ms |
| `fill` ohne Mischart | überall dasselbe Wort, zeilenweise | 64,6 → 1,3 µs (pygame 4,4 µs) |
| `to_rgba_bytes`, `setze_alle` | Standardformate zeilenweise statt `set_at` je Pixel | `tobytes` und alle `transform`-Ausgaben |

**2048 bei 1280×960: 38,8 → 3,1 ms je Bild.** Alle 14 Audits bestehen
seitdem auf rustgame, mitsamt ihren Zeitbudgets.

### Text: von 0 auf 11.984

Die Grenze hieß `font.metrics glyphenbox` — „höchstens 1 daneben,
Rasterkante". Gemessen an jedem druckbaren Zeichen der vier Schriften,
die PyGameZ benutzt, in 14 Größen, normal und fett: **0 von 11.984**
Zeichenbildern waren gleich. Die Größen stimmten fast immer, die Pixel
nie.

Ein `H` in Bahnschrift 22 zeigte warum: bei pygame ist die Oberkante eine
volle Zeile 255 und der Querbalken liegt auf genau zwei vollen Zeilen.
SDL_ttf lädt jede Glyphe mit `FT_LOAD_DEFAULT | FT_LOAD_TARGET_NORMAL` —
also **gehintet**, über FreeTypes TrueType-Interpreter (v40) —, und
rastert mit FreeTypes `ftgrays`. rustgame rasterte ungehintet mit
fontdue.

Drei Stücke, und jedes hat seinen eigenen Fund gebracht:

1. **`ftgrays.rs`** — FreeTypes Graustufen-Rasterer, **übertragen** aus
   2.11.1 (der Fassung in pygame-ce): `FT_Outline_Decompose`, die
   Zellenarbeit, `ft_glyphslot_preset_bitmap`, `ft_smooth_render` samt
   vierfacher Überabtastung für überlappende Konturen. Welche Fassung gilt,
   steht unter `#if`: unter MSVC x64 die `gray_render_line` mit
   `FT_UDIVPREP` und die Bézier-Bögen über Differenzen (`BEZIER_USE_DDA`).
2. **`hinting.rs`** — der gehintete Umriss über `skrifa` (Google
   fontations, reines Rust, `forbid(unsafe_code)`), dessen Interpreter
   gegen FreeTypes v40 abgeglichen ist. Damit: **11.953 von 11.984**.
3. **`gestalt.rs`** — die Zeile über `harfrust` (die HarfBuzz-Übertragung
   des HarfBuzz-Teams), so wie SDL_ttf sie mit `hb_shape` setzt.

Die letzten Fehler, und keiner war, wonach er aussah:

* **Das `$` in Bahnschrift** hat bei pygame seinen Strich nur oben und
  unten. Kein Rasterfehler, sondern eine GSUB-Ersetzung über
  `FeatureVariations` — und HarfBuzz wertet deren Bedingungen **auch in der
  Grundeinstellung** aus (fehlende Achsen zählen als 0), `read-fonts`
  dagegen nie. Bahnschrifts einziger Eintrag gilt beiderseits der 0.
* **`T-SPIN`, `MEGA-GEWINN!`**: GPOS hebt den Bindestrich zwischen
  Großbuchstaben um zwei Zeilen an (`y_offset`). Die eigene Zeilenrechnung
  kannte keine senkrechte Verschiebung. Jetzt kommen Glyphen **und** Lagen
  von HarfBuzz, mit FreeTypes gehinteten Vorschüben als Fontfunktion und
  HarfBuzz' eigener Skala — genau `hb_ft`.
* **Consolas fett `g`, ein Pixel 174 statt 173.** Mein Rasterer, gefüttert
  mit FreeTypes **eigenen** Punkten (aus pygames `freetype.dll` über
  `ctypes`), traf 173; `skrifa`s Segmente waren Punkt für Punkt gleich.
  Der Fehler lag in meiner Brücke: ich hatte die Punktliste aus dem
  Zeichenstift **erraten** und einen echten Punkt weggelassen, der genau
  auf dem Mittelpunkt seiner Nachbarn lag. Das ist nur ohne Verschiebung
  gleichgültig — FreeType verschiebt den Umriss **vor** dem Zerlegen, und
  `(a + b) / 2` schneidet bei negativer und positiver Summe verschieden ab.
  Jetzt kommen Arten und Konturenden aus dem ungehinteten Umriss, und die
  gehinteten Koordinaten werden im Gleichschritt zugeordnet. Nichts mehr
  geraten.

| | vorher | jetzt |
|---|---:|---:|
| Zeichenbilder (4 Schriften × 14 Größen × 2 × 107 Zeichen) | 0 / 11.984 | **11.984 / 11.984** |
| Sätze aus PyGameZ' Sprachdateien (4 Sprachen, 4 Schriftbilder) | — | **6.000 / 6.000** |
| PyGameZ mit **echtem** Text, 46 Spiele × 300 Bilder | nur Pong gemessen: 0 von 30 Bildern | **46 / 46** |

### Kursiv und die Stilflaggen

Nach dem Umbau auf gehintete Umrisse scherte rustgame Kursiv noch am
fertigen Graubild — und `freetype_test.test_freetype_Font_get_rect` fiel
um. SDL_ttf schert den **gehinteten Umriss** mit `FT_Outline_Transform`
(`xy = 0x366A`) und rastert danach. Dazu zwei Regeln von `TTF_SetFontStyle`
und `TTF_GetFontStyle`: auf einem Schnitt, der schon fett oder kursiv ist,
wird der Wunsch **gestrichen**, und `get_bold()` meldet den Stil des
Schnitts mit (FreeTypes `style_flags`: OS/2 `fsSelection`, sonst
`head.macStyle`). `Font("consolab.ttf").get_bold()` ist darum wahr, ohne
dass jemand `set_bold` gerufen hätte. Beides ist jetzt eine ständige
Probe: `font.render Kursiv` (2.280 Zeichenbilder, künstlich und auf echten
Kursivschnitten) und `font Stilflaggen`.

### Die Schrift, die nur zufällig da war

`pygame.init()` setzt die Schriftschalter, sagte der Schriftkiste aber
nicht, **wo die Hülle liegt** — das taten nur `font.init()` und
`freetype.init()`. Danach suchte `Font(None, 20)` die mitgelieferte
`freesansbold.ttf` am Arbeitsverzeichnis entlang. Aus `%TEMP%` gestartet:

```text
error: freesansbold.ttf not found - set RUSTGAME_DEFAULT_FONT
```

Im Projektordner fiel es nie auf, weil die Suche am Ende in **pygames**
Paket in `.venv` fündig wurde. Ohne installiertes pygame wäre das ein
Absturz gewesen — bei einer Engine, deren Zweck es ist, pygame zu ersetzen.
Zwei Ursachen, beide behoben:

* Der Ordner wird jetzt auch in `pygame.init()`, `get_fonts`, `match_font`
  und `freetype.SysFont` gesichert.
* `tools/einbauen.py` legte nur die `.pyd` nach `site-packages`. pygame hat
  `freesansbold.ttf` und `pygame_icon.bmp` **neben** seinem Modul; jetzt
  legt das Werkzeug beide mit und prüft sie wie die `.pyd`.

### `sysfont`: eine Grenze, die vier Fehler deckte

Die Grenze hieß „zwei Schriftnamen von 391 verlieren ihr `ä`“ — und meldete
**sechs** Abweichungen. Weil eine Grenze nur am Namen der Probe hing, sah
niemand die anderen vier:

| Fund | Ursache |
|---|---|
| `freesansbold` fehlte in `Sysfonts` (390 statt 391) | der Schriftordner, siehe oben — und pygame trägt den Eintrag **immer** ein, auch wenn die Datei fehlt |
| `initsysfonts()` gab `Sysfonts` zurück | pygame gibt `None`. Die drei Plattformsuchen reichten `initsysfonts()` durch; pygames lesen jedes Mal neu und geben ein eigenes Wörterbuch — jetzt wörtlich übertragen, samt `fc-list`-Warnung |
| die Schnitte standen sortiert | pygame trägt sie in Registry-Reihenfolge ein, und wo es „irgendeinen“ Schnitt nimmt, ist das der **erste**. Hier stand eine `HashMap` — deren Reihenfolge würfelt Rust **je Prozess neu** |

Dazu `create_aliases` mit einer erfundenen Tabelle (`sans-serif`, `cursive`,
`fantasy` — Namen, die pygame nicht kennt) statt pygames fünf Gruppen; ein
Zweitname zeigt bei pygame auf **dasselbe** Wörterbuch wie die gefundene
Schrift. Die Probe misst jetzt die ganze Namensmenge, die Reihenfolge,
`match_font` für jeden Namen in jedem Schnitt und die drei Plattformsuchen.
**Die Grenze `sysfont` ist gefallen.**

Und das Messgerät hatte zwei Lücken: ihr Kommentar sagte „verglichen wird
nicht die Reihenfolge“, verglichen wurde aber `repr` — und `gegenueber`
sortierte Wörterbücher nach Schlüsseln und zeigte darum **keine einzige
Zeile**, während die Probe rot war. Jetzt meldet es Reihenfolge-Unterschiede,
und unter allem liegt ein Netz: ist `repr` verschieden und findet sich
kein Einzelbefund, sagt es das.

### Ein Abdruck für jede Grenze

Der `sysfont`-Fund war kein Einzelfall. Jede der 16 Grenzen wurde gegen
ihren Vermerk gelesen — und vier deckten mehr, als sie sagten:

| Grenze | Vermerk | gemessen |
|---|---|---|
| `sysfont` | 3 Einträge | 6 |
| `Klassendoppel` | 2 Namen (`__release_buffer__`) | 4 — dazu `Cursor.__deepcopy__` und `ScancodeWrapper.__getattr__`, beide überflüssig und jetzt weg |
| `Fehlertexte` | 4 Einträge, darunter drei `XMP:`-Sätze | die drei gab es nicht mehr; dafür `midi.get_device_info("x")` und `music.get_metadata` ohne Arbeitsverzeichnis im Satz — beide behoben |
| `unbekanntes Schlüsselwort` | `tf_scale_by()` | `scale_by()`, und ein zweiter Fall, der zählt statt zu nennen |

Jetzt trägt jede Grenze einen **Abdruck ihrer Abweichung**
(`GRENZEN_ABDRUCK`: SHA-256 über genau die Zeilen, die `gegenueber` zeigt).
Ändert sich die Abweichung, zählt die Probe als Abweichung — auch unter
demselben Namen. Gegenprobe: ein falscher Abdruck macht aus `[bekannt]`
einen Fund und den Lauf rot. Nachgetragen wird nur bewusst, mit
`--abdruecke`, nachdem `--genau` gelesen ist.

### MIDI: Gerät Nummer 0

Die Probe `midi.Geraeteliste` ließ die Anzahl weg — „eine benannte Grenze:
pygame zählt 2, rustgame 1“. In der Grenzentafel stand dazu **nichts**.
Und es war nicht nur die Zahl: **Nummer 0 war ein anderes Gerät** — bei
pygame der `Microsoft MIDI Mapper`, hier der Wavetable-Synthesizer. PortMidis
`pm_winmm_init` trägt zuerst den Mapper ein (`MIDI_MAPPER`, kein
nummeriertes Gerät — `midir` zählt ihn nicht), dann Eingänge, dann
Ausgänge. Das ist jetzt nachgebaut; der Mapper führt wie bei `midimap.dll`
auf das Gerät aus `MIDIMap\szPname`, sonst auf das erste.
`get_device_info` wandelt sein Argument wie Cython (`1.5` ist Gerät 1,
Text gibt `an integer is required`, `2**31` einen `OverflowError`).

Nicht nachgebaut ist ein Fehler in pygame: nach `Output.close()` lässt sich
in derselben Sitzung **kein** Ausgang mehr öffnen (derselbe gibt
`Invalid device ID`, ein anderer `midi Output not open.`).

### Zwei WAV-Leser, nicht einer

pygame hat zwei: `mixer.Sound` liest über SDLs `SDL_wave.c`,
`mixer.music` über SDL_mixers `music_wav.c` — andere Reihenfolge, andere
Sätze, auch andere Annahmen. Hier stand **ein** Leser mit deutschen
Sätzen (`WAV ohne fmt-Block`). Gemessen und übertragen:

| Fall | `Sound` | `music` |
|---|---|---|
| kein `fmt ` | `Missing fmt chunk in WAVE file` | `Bad WAV file (no FMT chunk)` |
| `data` vor `fmt ` | `fmt chunk after data chunk in WAVE file` | geht |
| 300 Spuren | `Number of channels exceeds limit of 255` | `Invalid source channels` |
| Rate 0 | `Invalid sample rate` | `Source rate is equal to or less than zero` |
| Float mit 64 Bit | `64-bit IEEE floating-point format not supported` | geht |
| `b"RIFF"` | `Could not read RIFF header` | `Couldn't read first 12 bytes of audio data` |
| kaputtes OGG, FLAC, MP3, XM, WavPack | `Unrecognized audio format` | der Satz des Dekodierers |

Dazu **A-law und mu-law**: SDL dekodiert beide, hier stand „nicht
unterstützt“. Gemessen an allen 256 Bytewerten sind SDLs Werte genau die
G.711-Formeln. `music.load`, `queue` und `get_metadata` nehmen `bytes` nur
als **Pfad** (`bytearray` und `memoryview` sind `not a file object`); hier
wurden rohe Bytes als Dateiinhalt gespielt. Und ein **Ordner** als Pfad gab
`No file 'rustgame' found: Zugriff verweigert (os error 5)` — `exists()`
ist für einen Ordner wahr, und die Anzeige eines `io::Error` ist der Satz
des Betriebssystems in **dessen** Sprache.

Die Sätze der Musik-Dekodierer (`stb_vorbis_open_rwops: VORBIS_invalid_setup`,
`music_drflac: corrupt flac file (bad stream).`, auch `Out of memory`)
hängen an den Innereien von fünf C-Bibliotheken — eine benannte Grenze.

### Bilder: Nachsicht und Strenge, beide gemessen

Ein 8-Bit-TGA mit Echtfarbe brachte rustgame zum **Absturz**
(`index out of bounds`, als `PanicException` bis nach Python).
`IMG_tga.c` ist jetzt übertragen, und dabei zeigte sich, wie verschieden
SDL_images Leser sind:

* **TGA wird nur mit dem Typ `tga` gelesen** — es ist SDL_images einziges
  Format ohne Kennung. `load(BytesIO(tga))` ist `Unsupported image format`,
  `load(BytesIO(png), "x.tga")` ist `Unsupported TGA format`. Hier wurde
  TGA am Inhalt geraten und der Hinweis übergangen (`let _ = namehint;`).
  Und: `load("a.tga")` geht, `load("a.tga", "")` nicht — ein angegebener
  Hinweis gewinnt, auch ein leerer.
* **TGA liest ohne Prüfung.** Was fehlt, bleibt null, ein halbes Pixel wird
  halb geschrieben, und bei RLE wiederholt sich nach dem Ende das letzte
  Paket — gemessen `302070` aus dem Pixel `302010` und einem einzelnen
  Byte `70`.
* **PNM kann kein 16 Bit**, und ein kleiner Höchstwert skaliert mit
  `Uint8`-Überlauf: bei Höchstwert 15 wird 32 zu `544 & 0xFF`, also 32.
* **PCX und LBM sind streng**: `file truncated`, `error reading BODY chunk`.
  PCX kennt nur drei Arten (2- und 4-Bit-PCX sind `unsupported PCX
  format`), seine Tafel hat `1 << (Bits·Ebenen)` Einträge, und bei 8 Bit
  sucht SDL das Byte 12 **hinter den Pixeldaten**.

Und ein Fund, der jede Fläche betrifft: **`image.tobytes` einer 15- oder
16-Bit-Fläche** gibt bei pygame `f8` für Weiß, `get_at` dagegen 255.
`tobytes` schiebt nur (`((v & Rmask) >> Rshift) << Rloss`), `get_at` geht
über `SDL_GetRGBA` und wiederholt die Bits. Hier kam beidemal 255 heraus.

Zwei Sorten pygame-Verhalten sind **nicht** nachzubauen und stehen in
keiner Probe: ein GIF ohne Bild und ein kaputtes TIFF melden den
**liegengebliebenen** Satz des vorigen Fehlers (`JPEG loading error`,
`Surface doesn't have a colorkey`), und eine TGA-Tafel über das Dateiende
liest nicht vorbelegten Speicher (im frischen Prozess Nullen, im langen
Lauf 37, 165, 232).

### 36 deutsche Sätze, die der Wächter nicht sah

`deutsche_fehlertexte.py` meldete 0, und im Mixer stand `WAV ohne
fmt-Block`. „ohne“ stand in keiner seiner Listen, und `zu wenig` traf
`zu wenige` nicht. Ein Suchlauf mit deutschen Funktionswörtern fand **30**
weitere Stellen, das Lesen danach noch sechs (`zu wenige`, `unvollstaendig`,
`unbrauchbare`) - **36** in allen — in den Bildformaten, im WAV-Leser, bei WavPack, in der
Anzeige (`weder minifb noch das fensterlose Stück machen einen GL-Kontext
auf`). Alle sind jetzt englisch, wo messbar mit SDLs Wortlaut. Die
Wortliste ist erweitert, und eine Gegenprobe prüft sie: 18 von 18 alten
Sätzen gefunden, keiner der englischen.

### Das Messgerät schrieb in fremden Speicher

Jede Ausgabe von `vergleich_module.py` trug zweimal

```text
SystemError: deallocated bytearray object has exported buffers
```

— in **beiden** Engines, darum fiel es im Vergleich nie auf. Die Probe
`Puffer spaet lesen` nahm als Datenadresse `id(bytearray) +
bytearray.__basicsize__ - 1`. Das ist nicht der Inhalt, sondern das letzte
Byte des **Kopfes** — unter CPython 3.14 das obere Byte von `ob_exports`.
`.raw` las von dort, drei Bytes davon **hinter** dem Objekt, und
`write(b"\x09", 0)` schrieb in den Zähler. Jetzt ist es ein eigener
Speicher von `ctypes`. *Ein Messgerät, das fremden Speicher beschreibt,
misst Glück.*

### Was nach Runde 18 offen war

* `freetype.Textbreite`: die Schreibmarke in `freitext.rs` setzt anders als
  pygames `ft_layout` („Hallo Welt“ +6). — **Erledigt in Runde 19.**
* **AIFF** liest rustgame nicht (pygame auf beiden Wegen), LBM vom Typ
  `PBM ` ebenso nicht. — **Erledigt in Runde 19.**
* Einfarbiger Text und Konturen laufen noch über fontdue statt
  `ftraster.c` und `FT_Stroker`. — **Erledigt in Runde 19** (und dabei
  zeigte sich: einfarbig war das Bild **immer leer**).
* Text-Rendern ist langsamer als bei pygame; das Schach-Audit braucht viel
  länger.
* **Ein unerklärter Absturz**: einmal brach `vergleich_module.py` in der
  ersten Sekunde mit `Segmentation fault` ab. Drei Wiederholungen mit
  demselben Aufruf liefen sauber durch; die Ursache ist nicht gefunden.

### Die Abnahme nach Runde 18

| Messung | Ergebnis |
|---|---|
| Modulvergleich | **840** Proben, **0** Abweichungen, **18** benannte Grenzen, jede mit Abdruck |
| pygames eigene Testsuite | **2364 / 0 / 0 / 121**, dieselben 121 übersprungen, Name für Name |
| PyGameZ, alle 46 Spiele, je 300 Bilder, echter Text | **46 von 46** gleich, und zweimal gefahren |
| PyGameZ' 14 Audits auf rustgame | **14 von 14** bestanden, samt Zeitbudgets; Nutzerdaten per SHA-256 unverändert. Langsamer sind Casino (399 s gegen 113 s) und Schach (252 s gegen 30 s); beim Schach ist es das Anlegen vieler Schriftgrößen (gemessen), beim Casino ist die Ursache nicht gemessen |
| je gerufene Namen | **775 von 921** (84 %), unverändert; der Rest ist gerätegebunden |
| Zustand vor `init()` und nach `quit()` | **360** Fragen, 0 Abweichungen |
| halbfertige Objekte | **463** Fragen an 51 Klassen, 0 Abweichungen |
| API-Namen | **3471 / 3471** |
| deutsche Fehlertexte | **0** — mit erweiterter Wortliste und Gegenprobe |
| Rust | **668** Tests im Workspace und **12** in der Hülle grün, Clippy und Doku ohne Warnung in beiden |

## Runde neunzehn: was keine Probe zeichnete

Runde 18 hatte eine Liste hinterlassen: `freetype.Textbreite`, AIFF und
LBM `PBM `, einfarbiger Text und Konturen noch über fontdue. Die Runde hat
die Liste abgearbeitet und dabei eine Regel bestätigt, die schon zweimal
galt. **Ein Zweig, den keine Probe betritt, ist nicht gleich, sondern
ungeprüft.** Fast jeder Fund dieser Runde lag in einem Weg, der „hier
nicht nachgebaut, also der alte Weg“ sagte, und der alte Weg war in keinem
einzigen Fall gleich.

### Einfarbiger Text war immer leer

`font.render(text, False, farbe)` gab hier **seit jeher eine leere
Fläche**. Die Einbit-Fläche wurde gebaut, dann mit `set_palette`
umgestellt, und dabei wurden die Pixel gegen die alte, leere Tafel
umgerechnet. pygames Testsuite prüft dort nur Größen, keine Probe verglich
Pixel. Aufgefallen ist es erst beim Übertragen von FreeTypes
Schwarzweiß-Rasterer.

* **`ftraster.c`** ist jetzt übertragen (`ftraster.rs`), samt
  Dropout-Steuerung, Bandteilung und den Scan-Marken, die FreeTypes
  TrueType-Lader in den Umriss schreibt. Gegen pygames eigene
  `freetype.dll` über `ctypes` gemessen: **17.160 von 17.160** Bildern
  gleich. Den Zustand des Interpreters nach `prep` (`scan_control`,
  `scan_type`) hält skrifa privat; dafür trägt `vendor/skrifa` einen
  kleinen Zusatz.
* **`FT_Stroker`** (`ftstroke.rs`, mit `fttrigon.c`) für
  `font.set_outline`: **27.456 von 27.456** Umrissen gleich. Dazu SDL_ttfs
  Maße einer Konturschrift (`TTF_Size` +2n, Auflänge +2n, Unterstrich).

### `pygame.freetype` setzt jetzt selbst

Die Hülle setzte `pygame.freetype` über SDL_ttfs Zeile und schnitt das
Ergebnis zu. Jetzt ist pygames eigener Satz übertragen (`ft_layout.c`,
`ft_render.c` → `ftzeile.rs`): Schreibmarke, Abschluss-Leerzeichen,
Kasten, Kerning nur aus der alten `kern`-Tabelle, und die vier Stile wie
pygame (`FT_Outline_Embolden`, Scherung 0,22, `FT_Bitmap_Embolden`,
Unterstrich mit Bruchkanten). Die Grenze `freetype.Textbreite` ist
gefallen, und mit ihr Fälle, die nie benannt waren:

| Fall | vorher | jetzt |
|---|---:|---:|
| gedreht (`rotation`) | 0 von 24 | 28 von 28 |
| senkrecht (`vertical`), vier Stile, glatt und einfarbig | 0 von 3 | 56 von 56 |
| Bruchgrößen und andere Auflösung | 0 von 8 | 1050 von 1050 |
| Bitmapschriften, eingebettete Bitmaps, CFF | 539 von 918 | 918 von 918 |

Drei Einzelheiten aus dem Weg dorthin:

* **Senkrechte Lager brauchen den Vorschub vor dem Runden.** FreeType
  rechnet `vertBearingX = horiBearingX − horiAdvance / 2` mit dem
  ungerundeten Vorschub; skrifa gibt ihn nur gerundet heraus. Ein zweiter
  Zusatz in `vendor/skrifa` (`vorschub_roh`) hält ihn fest.
* **pygame setzt `FT_LOAD_VERTICAL_LAYOUT` nie.** Bei TrueType ändert das
  nichts, bei CFF alles: ohne das Flag sind die senkrechten Lager null.
  `cffgload.c` lag nicht vor; entschieden hat ein `ctypes`-Aufruf an
  pygames `freetype.dll`.
* **Bruchgrößen haben zwei Skalen.** Mit `head.flags` Bit 3 hintet
  FreeType mit der gerundeten Größe, meldet aber Auflänge, Unterstrich
  und Kerning mit der ungerundeten (`get_sized_ascender(20.5)` ist bei
  Arial 19, bei 21 sind es 20).

Und **`_PGFT_BuildRenderMode`** hatte hier vier Sätze nicht: gedrehter
Unterstrich, gedrehtes `wide` und senkrechter Unterstrich wurden
gezeichnet, wo pygame `ValueError` gibt. Für Bitmapschriften stand ein
erfundener Satz da, und `f.strong = False` warf bei pygame, hier nicht.

### CFF: kein einziges Bild

`.otf`-Schriften mit PostScript-Umrissen (CFF) gingen in **beiden**
Schriftmodulen ungehintet über fontdue. An `PyGameMono.otf` aus pygames
eigenen Testdaten war in `pygame.font` **keines** von 165 Bildern gleich,
an `test_fixed.otf` 8. skrifa hatte FreeTypes CFF-Hinter (`cf2`) die ganze
Zeit dabei. Es fehlte die Punktliste so, wie FreeTypes `ps_builder` sie
baut: Koordinaten mit `x >> 10` abgerundet, der letzte Punkt fällt, wenn
er auf dem ersten liegt. Jetzt **330 von 330**.

Dabei fiel ein falscher Vermerk: in `face.rs` stand, SDL_ttf sehe die
OS/2-Typo-Werte nie. FreeType 2.11.1 nimmt sie, sobald `fsSelection`
Bit 7 (`USE_TYPO_METRICS`) steht. Unter den Windows-Schriften hat keine
das Bit **und** abweichende Werte; gemessen ist es an einer im Speicher
gebauten Schrift. Und eingebettete Bitmaps werden in SDL_ttf **nicht**
kursiv geschert.

### Dateien

* **AIFF** und AIFC auf beiden Wegen (`load_aiff.c`, `LoadAIFFMusic`
  samt `sowt`, A-law, µ-law, `fl32`/`fl64`), **VOC** (`load_voc.c`),
  Trackermodule und die `XMP:`-Sätze wie libxmp.
* **LBM** ganz nach `IMG_lbm.c` (ILBM und PBM, EHB, HAM6/8, 24 Ebenen),
  **GIF** mit SDL_images eigenem LZW-Leser, **ICO/CUR**, **XV**,
  `IMG_isJPG`, TGA mit leerer Tafel. Die Grenze „Bilddateien kaputt“ ist
  gefallen.
* Eine 8-Bit-Fläche **mit Masken** (RGB332) bekam hier eine Farbtafel und
  suchte die nächste Tafelfarbe, statt Bits abzuschneiden.
* `Surface.get_colorkey()` gab eine `Color` statt eines Tupels. `==`
  versteckte das: `Color(1, 2, 3) == (1, 2, 3, 255)` ist wahr.

### Der Fehlgriff dieser Runde

Zwei neue Proben hatten ihre Untergrenze **über** dem echten Wert (900
gegen 744 und 581). Seit Runde 14 steht dafür ein Riegel in `main()`, und
er hat gegriffen: „Wächter gefallen“ statt „gleich“. Die Untergrenze wird
jetzt nach dem ersten Lauf gesetzt.

### Was bleibt

* **Größen mit zwei Achsen** in `pygame.freetype` (`size=(24, 12)`):
  FreeType hintet sie mit gestrecktem Interpreter, skrifa kennt nur eine
  Skala. Eine neue benannte Grenze mit Abdruck; jede **eine** Größe ist
  gleich.
* Text ist langsamer als bei pygame (Schach-Audit 250 s gegen 29 s, Casino-Arcade 351 s gegen 105 s).

### Die Abnahme nach Runde 19

| Messung | Ergebnis |
|---|---|
| Modulvergleich | **854** Proben, **0** Abweichungen, **17** benannte Grenzen, jede mit Abdruck |
| pygames eigene Testsuite | **2364 grün, 0 falsch, 0 Fehler, 121 übersprungen (von 2485)**, dieselben 121 |
| PyGameZ, alle 46 Spiele, je 300 Bilder | **46 von 46**, mit echtem Text **46 von 46** |
| PyGameZ' 14 Audits auf rustgame | **14 von 14** |
| je gerufene Namen | **775 von 921 (84 %)** |
| Zustand vor `init()` und nach `quit()` | **360 Fragen, 0 abweichend** |
| halbfertige Objekte | **463 Fragen an 51 Klassen, 0 Abweichungen** |
| API-Namen | **3471 / 3471** |
| deutsche Fehlertexte | **0** |
| Pixelbytes Programmvergleich | **0 von 76.800** und **0 von 9600** |
| Rust | **694** Tests (682 im Workspace, 12 in der Hülle), Clippy und Doku ohne Warnung in beiden |

## Runde zwanzig: zwei Achsen, und jede Schrift des Systems

Runde 19 hatte eine echte Lücke hinterlassen: `pygame.freetype` mit
**zwei Größenachsen** (`size=(24, 12)`). Die Runde hat sie geschlossen und
danach eine Frage gestellt, die in neunzehn Runden niemand gestellt hatte:
**Wie sieht es mit den Schriften aus, die keine Probe ausgesucht hat?**

### Zwei Achsen: FreeTypes gestreckter Interpreter

Bei `x_ppem != y_ppem` hintet FreeType mit einem **gestreckten**
TrueType-Interpreter. Die CVT wird mit der größeren Skala angelegt, jedes
Lesen und Schreiben geht durch `Current_Ratio()` (das Seitenverhältnis
entlang des Projektionsvektors), `MPPEM` und `DELTA*` sehen die gestreckte
Pixelzahl, und `MD`, `MDRP` und `IP` messen Abstände im Originalumriss mit
zwei Skalen. skrifa kannte eine Skala. Jetzt steckt der gestreckte Modus im
skrifa-Abzug (`HintingInstance::new_gestreckt`), samt FreeTypes
CORDIC-Länge `FT_Vector_Length`, gegen die DLL an 74 Vektoren geprüft.

Der Satz rechnet senkrecht mit `y_scale` (Auf- und Unterlänge,
Zeilenhöhe, Unterstrich) und waagerecht mit `x_scale` (Vorschub,
Kerning, Fettung). CFF ist der `x`-Wert aus der Breite und der `y`-Wert
aus der Höhe (`cf2_glyphpath_hintPoint`), und eine BDF-Schrift sieht nur
die Höhe (`BDF_Size_Request`).

| Fall | vorher | jetzt |
|---|---:|---:|
| zwölf Windows-Schriften, 10 Größenpaare, alle Stile | 0 von 18 (Probe) | **8760 von 8760** |
| pygames Testschriften (TTF, CFF, BDF) | — | **8176 von 8176** |

Auf dem Weg dorthin kamen fünf Funde, die mit zwei Achsen nichts zu tun
haben: `get_metrics` stimmte nur im Grundfall — an `Font(None, 24)` und
`"Ag"` waren fett, schräg, breit, einfarbig, 30 Grad, `20.5` und
`(24, 12)` jedes Mal anders; jetzt lädt es jedes Zeichen wie beim Satz
(`_PGFT_GetMetrics`), gemessen **4600 von 4600** gleich; bei kleinen Größen schaltet `prep` das Hinting ab, und
FreeType skaliert dann mit der **gerundeten** Größe und setzt die
Dropout-Flaggen trotzdem (92 von 200 Bildern waren anders); CFF rundet die
Skala nie; ein Strike gewinnt die Größenanfrage auch für `get_sized_*`;
und Bitmapschriften nehmen jede Größe, deren `FX6_ROUND` den Strike trifft
(`8.4` geht bei `PyGameMono-8.bdf`), mit dem Satz `Failed to resize font:
invalid pixel size` — hier stand ein erfundener. `freetype.get_error()`
gibt jetzt den letzten FreeType-Satz zurück wie pygame.

### Jede Schrift des Systems

`tools/alle_schriften.py` rendert alle 380 Dateien des Windows-
Schriftordners mit beiden Schriftmodulen auf beiden Engines. Der erste
Lauf fand **über 650 Abweichungen in über 30 Schriften** — keine davon war
mit den zwölf ausgesuchten Schriften zu sehen:

* **Calibris `kern`-Tabelle** hat 26.706 Paare in einer Teiltabelle;
  ihr Längenfeld ist 16 Bit und sagt 29.178. FreeType kürzt die Liste auf
  4860 Paare, ttf-parser las alle. `tt_face_load_kern` und
  `tt_face_get_kerning` sind übertragen.
* **Courier New fett** macht beim Hinting Kontrollpunkte zu
  Kurvenpunkten (`FLIPRGON`). Die Hülle ordnete den Stift-Strom den
  ungehinteten Punktarten zu, scheiterte, und **jede** Glyphe fiel still
  auf den alten Weg. Jetzt gibt der skrifa-Abzug den Umriss direkt als
  FreeTypes Punktliste heraus (`roh_umriss`).
* **Strikes** (Calibri 12–19, Courier New 4–21): der Vorschub steht in
  den sbit-Maßen, und senkrechter Satz nimmt bei kleinen sbit-Maßen
  `linearVertAdvance` (Calibri senkrecht bei 19: 85 Zeilen statt 13).
* **Symbolschriften**: SDL_ttf wählt die Zeichentabelle (3,0).
* **`monbaiti.ttf`**: das CV-Programm ruft eine nie definierte Funktion,
  FreeType lehnt jede gehintete Glyphe ab. pygame sagt dann `Couldn't
  find glyph`, `Text has zero width` oder `Unable to load glyph for id
  36` — rustgame zeichnete still. Und pygame.freetype **stürzt** nach
  einem solchen Fehler bei weiteren Aufrufen ab.

| Durchlauf | Fälle | anders |
|---|---:|---:|
| `pygame.freetype` | 4083 | **23** in 9 Schriften (dazu 2 Ladesätze von `.fon`) |
| `pygame.font` | 3061 | **26** in 9 Schriften |

Was bleibt, sind zwei benannte Grenzen: das Format **`.fon`** (189
Windows-Bitmapschriften, die pygame über FreeTypes `winfnt` lädt) und ein
**Rest** aus Hinting um 1/64 Pixel, einfarbigen Rasterpixeln und drei
Breiten (siehe unten).

### Zweimal falsch gelesen

Zweimal stand nach dem Lesen der FreeType-Quelle eine Ursache fest, und
beide Male war sie falsch: das Zurücksetzen bei `INSTCTRL` Bit 2 änderte
nichts, und das Graustufen-Bit von `GETINFO` verdarb Courier New bei 23,5
(der v40-Interpreter setzt es nie). Erst die Punkte aus pygames DLL neben
skrifas Punkten zeigten, was los war. *Ein Orakel schlägt Lesen.*

### Die Abnahme nach Runde 20

| Messung | Ergebnis |
|---|---|
| Modulvergleich | **859 Proben, 0 Abweichungen, 16 benannte Grenzen, jede mit Abdruck** |
| pygames eigene Testsuite | **2364 grün, 0 falsch, 0 Fehler, 121 übersprungen (von 2485)**, dieselben 121 |
| PyGameZ, alle 46 Spiele, je 300 Bilder | **46 von 46**, mit echtem Text **46 von 46** |
| PyGameZ' 14 Audits auf rustgame | **14 von 14** |
| je gerufene Namen | **775 von 921 (84 %)** |
| Zustand vor `init()` und nach `quit()` | **360 Fragen in drei Zuständen, 0 Abweichungen** |
| halbfertige Objekte | **463 Fragen an 51 Klassen, 0 Abweichungen** |
| API-Namen | **3471 / 3471** |
| deutsche Fehlertexte | **0** |
| Pixelbytes Programmvergleich | **0 von 76.800** und **0 von 9600** |
| Rust | **694 (682 im Workspace + 12 in der Hülle)** |

Text bleibt langsamer als bei pygame; die Zeitbudgets der Audits halten
(Schach-Audit 263 s gegen 31 s, Casino-Arcade 404 s gegen 110 s).

## Runde einundzwanzig: die Frage vor dem Commit

Diesmal kam die Frage nicht aus pygame, sondern vom Nutzer: **Ist alles
da, was man braucht, um rustgame so zu benutzen wie pygame-ce?** Drei
Messungen, und jede fand etwas.

### Ein frischer Klon

Genau die Dateien, die ein Commit von `rustgame/` mitnähme, kamen in einen
leeren Ordner. Dort wurden sie gebaut und in eine leere venv **ohne
pygame-ce** gelegt (`einbauen.py`). Das geht, in anderthalb Minuten.
Unterwegs lagen fünf Dateien im Quellbaum, die der Commit mitgenommen
hätte:

* eine `rundlauf.png`, die `vergleich_module.py` bei **jedem** Lauf neben
  sich schrieb (jetzt im Temp-Ordner),
* eine `argname` mit Systemdaten aus einer längst ausgenommenen Probe,
* eine `temp_file.bmp`,
* eine leere `deutsch2.json`,
* eine `tools/vorlage/SDL_audiocvt_channels.c`, in der nur
  `404: Not Found` stand.

### Ein gewöhnliches pygame-Programm

Es gab keinen Weg, ein **unverändertes** pygame-Programm auf rustgame zu
starten, außer über die Messwerkzeuge. Jetzt gibt es `tools/starten.py`.
Es benutzt dieselbe Umleitung wie die Testsuite und PyGameZ. Sein erster
Lauf fand drei Fehler, die zwanzig Runden lang niemand sah, weil jede
Messung sie gleich mitbrachte:

* **`pygame.__name__` hieß `pygame.cursors.pygame`.** Die Umleitung hängte
  jedes Modulfeld eines Untermoduls ein, auch `os`, `sys` und das Paket
  selbst (`cursors.pygame`). Umbenannt wurde nach dem Schlüssel, und der
  letzte gewann. 32 Einträge waren falsch; jetzt sind es zwei Aliase
  (`pygame.mixer.music`, `pygame.ftfont.draw`) mit richtigem Namen.
* **Das Fenster war immer `dummy`.** Die Testsuite stellt das beim Laden
  ein, und das Startwerkzeug erbte es.
* **pygames Grußzeile beim Import fehlte** (`pygame-ce 2.5.7 (SDL 2.32.10,
  Python 3.14.4)`). Jede Messung schaltet sie mit
  `PYGAME_HIDE_SUPPORT_PROMPT` ab. Jetzt ist sie Byte für Byte gleich.

Danach lief das Programm mit echtem Fenster (minifb, 60 Bilder je
Sekunde), Ereignissen, Ton, `font`, `SysFont` und `freetype`.

### Eine 24-Bit-Fläche

Ein Querschnitt durch die Module (`mycache/r21/direkt.py`) war gleich bis
auf `image.save`. Die Spur führte aber nicht zum Speichern, sondern zum
**Blit davor**: Alpha auf ein Ziel mit weniger als 32 Bit mischt anders.
Jede Probe, jedes Referenzbild und jeder Fuzzer-Lauf blittete auf 32 Bit.
Ein Ziel mit 24 Bit liefert aber jedes `image.load` einer JPEG-Datei.

| Ziel | Pixelalpha | Flächenalpha | Schlüssel + Alpha | `BLEND_RGBA_ADD` |
|---|---|---|---|---|
| 32 Bit | gleich | gleich | gleich | gleich |
| 24 Bit | bis 1 | bis 1 | gleich | gleich |
| 16 und 15 Bit | gleich | bis 8 | gleich | gleich |
| 8 Bit mit Farbtafel | bis 150 | bis 34 | bis 69 | gleich |

Und JPEG selbst: rustgame liest mit `zune-jpeg`, pygame mit
libjpeg-turbo. **39 von 41** Dateien hatten andere Pixel, um 2 bis 61
Stufen. Geschrieben wurde mit der Güte 75, „gemessen an der
Dateigröße“. pygames `imageext.c` sagt `#define JPEG_QUALITY 85`, und die
Quantisierungstafel in pygames Datei sagt es auch. Die Güte ist
berichtigt, der Kodierer bleibt ein anderer. PNG schreibt dieselben Pixel
mit anderer Kompression.

Die drei sind jetzt Proben mit Abdruck (`Alpha-Blit unter 32 Bit`, `JPEG
lesen`, `JPEG und PNG schreiben`) und **die nächste echte Arbeit**: SDLs
Blitter (`SDL_BlitA.c`) und libjpeg-turbos Dekodierer übertragen.

### Die Lizenz

Der Lizenzabschnitt nannte nur pygame. Seit den Schriftrunden stecken
aber FreeType-Übertragungen darin (FTL oder GPL 2). Jetzt steht unten die
Herkunft samt FreeTypes Vermerk, und die Frage, ob das so veröffentlicht
werden darf, ist als offen benannt.

### Die Abnahme nach Runde 21

| Messung | Ergebnis |
|---|---|
| Modulvergleich | **862 Proben, 0 Abweichungen, 19 bekannte Grenzen** |
| pygames eigene Testsuite | **2364 grün, 0 falsch, 0 Fehler, 121 übersprungen (von 2485)**, dieselben 121, Name für Name |
| PyGameZ, alle 46 Spiele, je 300 Bilder | **46 von 46**, mit echtem Text **46 von 46** |
| PyGameZ' 14 Audits auf rustgame | **14 von 14**, Nutzerdaten unverändert (Schach 293 s gegen 34 s, Arcade-Casino 428 s gegen 126 s) |
| jede Schrift des Systems | `pygame.freetype` 4082 Fälle, 24 anders (dieselben 23 wie in Runde 20 und ein Ladesatz von `script.fon`; pygame stürzte diesmal auch bei `roman.fon` ab), `pygame.font` 3061 Fälle, 26 anders |
| je gerufene Namen | **775 von 921 (84 %)** |
| Zustand vor `init()` und nach `quit()` | **360 Fragen in drei Zuständen, 0 Abweichungen** |
| halbfertige Objekte | **463 Fragen an 51 Klassen, 0 Abweichungen** |
| API-Namen | **3471 / 3471** |
| deutsche Fehlertexte | **0** |
| Pixelbytes Programmvergleich | **0 von 76.800** und **0 von 9600** |
| frischer Klon in leerer venv | gebaut, eingebaut, Querschnitt gleich bis auf die drei neuen Grenzen |
| Rust | **694 Tests (682 + 12), clippy 0/0, doc 0/0** |

## Zweiunddreißig benannte Grenzen — neunzehn davon als Probe

`tools/vergleich_module.py` zählt sie **gesondert**: der Lauf wird grün,
die Abweichung bleibt sichtbar, und eine *neue* fällt weiterhin auf.
Dasselbe Verfahren wie beim Fuzzer.

**Neunzehn** melden sich so, als `[bekannt]` in der Ausgabe — und seit
Runde 18 jede mit einem **Abdruck** ihrer Abweichung: ändert sich die
Abweichung, ist es ein Fund, auch unter demselben Namen. Sieben nicht,
und alle sieben aus demselben Grund — sie stecken in einem Wert, der sonst
jeden anderen Unterschied mitverdecken würde, oder sie lassen sich gar
nicht vergleichen:

* **`pygame.imageext` am Hauptmodul** ist in den zwei `dir()`-Proben beim
  Namen ausgenommen.
* **Felder an der Klasse** ist der Grund, warum `Klassennamen` die
  **rufbaren** Namen vergleicht und nicht alle.
* **Wege ins Paket und die Fassung** (`sysfont.pygame_main_file`,
  `docs.PKG_DIR`, `ver`, `vernum`, `rev`, `SDL`) sind in
  `Konstantenwerte` beim Namen ausgenommen: eine einzelne `.pyd` hat
  keinen Paketordner, und die Fassung sagt, *was* das Paket ist.
* **Der Wortlaut der Rekursionswächter** wird in
  `Methoden falsch rufen` zur Marke `<rekursion>`, wie der der
  Argumentleser zu `<argumentfehler>`.
* **`__doc__` der Rufbaren und Felder** — 530 ohne Satz, 160 Felder, 1063
  mit deutscher Prosa. `Dokumentation` vergleicht darum Module und
  Klassen (dort ist `__doc__` schreibbar, und sie tragen pygames Satz
  wörtlich).
* **`inspect.signature`** — pygame hat an 662 von 2018 Stellen keine;
  `Signaturform` vergleicht darum nur, wo pygame eine hat, und dort die
  **Form** ohne die Namen.
* **Was pygame nicht festhält** — ein liegengebliebener SDL-Fehler als
  Satz und nicht vorbelegter Speicher als Farbe. Beides hängt an dem, was
  vorher geschah, und steht darum in keiner Probe.

Dazu kommen vier, die **kein** Probenfeld haben, weil sie nur am
halbfertigen Objekt sichtbar sind — `tools/halbfertig.py` führt sie
einzeln und nennt sie `[bekannt]`.

Die Zahl im Lauf ist also 19, gezählt sind 32 (in der Tabelle teilen sich manche eine Zeile), und das ist kein
Widerspruch, sondern der Unterschied zwischen „weicht ab", „ist
ausgenommen" und „ist der Grund für die Form der Probe".

| Grenze | warum |
|---|---|
| `gl_set_attribute` | pygame nimmt den Wunsch stillschweigend an, rustgame lehnt ab. Es gibt keinen GL-Kontext, auf den er je wirken könnte; ein `None` hieße „gemerkt, wird gemacht". |
| Windows-Bitmapschriften `.fon` | Seit Runde 20 gemessen (`tools/alle_schriften.py`): 189 Dateien lädt pygame über FreeTypes `winfnt`, rustgame nicht. Machbar: das NE-Format und seine FNT-Ressourcen auf die vorhandene Bitmapschrift abbilden. |
| Rest aus dem Durchlauf über alle Schriften | 23 von 4083 Fällen (`pygame.freetype`) und 26 von 3061 (`pygame.font`): Hinting um 1/64 Pixel bei Palatino und Arial fett kursiv, einfarbige Rasterpixel bei fünf Schriften, drei Breiten. Vorher über 650 Fälle in über 30 Schriften. |
| Alpha-Blit unter 32 Bit | Seit Runde 21, als Probe mit Abdruck. Auf ein Ziel mit 24 Bit mischt rustgame Pixel- und Flächenalpha um eine Stufe anders, auf 16 und 15 Bit das Flächenalpha um eine Kanalstufe, auf 8 Bit mit Farbtafel jede Art mit Alpha weit daneben. `BLEND_RGBA_ADD` und alles auf 32 Bit stimmen. pygame mischt dort mit SDLs eigenem Blitter (`SDL_BlitA.c`); der ist zu übertragen. |
| JPEG lesen | Seit Runde 21, als Probe mit Abdruck. `zune-jpeg` gegen libjpeg-turbo: 39 von 41 Dateien mit anderen Pixeln, 2 bis 61 Stufen. Abhilfe ist libjpeg-turbos Dekodierer (`JDCT_ISLOW`, weiches Hochrechnen der Farbebenen, `ycc_rgb_convert`), übertragen. |
| JPEG und PNG schreiben | Seit Runde 21, als Probe mit Abdruck. PNG: dieselben Pixel und Blöcke, andere Kompression. JPEG: `jpeg-encoder` gegen libjpeg-turbo mit Güte 85 (hier stand 75, aus der Dateigröße geschätzt). |
| `camera.get_backends` | pygame meldet hier `['_camera (MSMF)', 'OpenCV']`, rustgame `['_camera (MSMF)']`. Der zweite Eintrag ist kein Treiber, sondern ein **zweites Kamerastück**: pygame hat neben seinem C-Modul eine reine Python-Fassung über `cv2`, und die taucht auf, weil `opencv-python` in derselben Umgebung liegt. Ohne `cv2` gibt pygame denselben einen Eintrag. Ein OpenCV-Weg daneben wäre kein Zugewinn — er führt zum selben Gerät. |
| `__module__` der Typen | sagt `rustgame_py.surface`, wo pygame `pygame.surface` sagt — und das bleibt so, denn es ist die **Wahrheit**. Die Typnamen selbst stimmen alle, und die **Fehlermeldungen** schreiben `pygame.surface.Surface`, weil pygames eigene Tests darauf prüfen. |
| OGG anders gerundet | OGG/Vorbis liest hier `lewton`, bei pygame `stb_vorbis`. Zwei Dekodierer eines **verlustbehafteten** Formats runden verschieden: 51,2 % der 626.648 Werte sind gleich, die mittlere Abweichung ist **0,52** — also im Wesentlichen eine Eins im letzten Bit. Größer als 2 sind 178 Werte (0,03 %), und die liegen **alle an der Aussteuerungsgrenze**. |
| MP3 anders gerundet | Dieselbe Lage mit `symphonia` gegen SDL_mixers Dekodierer: 49 % gleich, Mittel **0,53**, größte Abweichung **2** — kein einziger Ausreißer. |
| `__release_buffer__` bei `Color` und `PixelArray` | PyO3 legt den Schlitz an, sobald eine Klasse `__releasebuffer__` umsetzt — und umsetzen **muss** sie es, weil `puffer::fuellen` die Zahlenfelder (`shape`, `strides`) auf dem Haufen anlegt und in `view.internal` parkt. Ohne den Haken bleibt bei **jeder** Puffersicht ein Kasten liegen. pygames C-Typen brauchen ihn nicht: ihre Zahlenfelder stehen im Objekt selbst. Zwei Namen gegen ein Leck. |
| `_render.Image` | hat bei pygame **keinen einzigen** öffentlichen Namen — die echte Klasse mit elf Namen heißt `_sdl2.video.Image`, und `_render.Image` ist ein leerer Stummel (auch `_render.Renderer` und `_sdl2.video.Renderer` sind dort **zwei** Objekte, mit 23 und 24 Namen). Hier ist es je eine Klasse. Einen leeren Zwilling daneben zu bauen wäre toter Quelltext für eine Zahl. |
| Felder an der Klasse | PyO3 legt `#[pyo3(get, set)]` als Beschreiber an der **Klasse** ab, pygame setzt dasselbe Feld im `__init__` an der **Instanz** — sechzehn Klassen, immer dieselbe Sorte Name (`lostsprites`, `dirty`, `ratio`). An der Instanz hat sie jede Seite. Darum vergleicht `Klassennamen` die **rufbaren** Namen. |
| `pygame.imageext` am Hauptmodul | pygame hat den Namen **nie** als Feld (sein `__init__.py` löscht ihn wieder); rustgame muss ihn haben, weil `rustgame_py` ein **einziges Modul** ohne `__path__` ist und `from pygame import imageext` sonst scheitert — drei Tests in `imageext_test.py`, nachgemessen. Genau **ein** Name, und jeder andere Überschuss am Hauptmodul fällt weiterhin auf. |
| `_sdl2` roher Tonstrom | `AudioDevice` und `set_post_mix` brauchen einen Rückruf **aus dem Tonfaden**. rustgames Mixer hat keine zentrale Mischstufe (jeder Kanal geht einzeln an die Tonkiste), und ein Rückruf aus dem Tonfaden müsste die Sperre des Auswerters holen — ein Tonfaden, der darauf wartet, verschluckt Abschnitte. Also ein klarer Fehler statt eines Hakens, der angenommen und nie gerufen wird. Nebenbei gemessen: pygame nimmt `set_post_mix(7)` an und **stürzt ab**, sobald der Haken gerufen wird. |
| unbekanntes Schlüsselwort | pygame sagt `this function got an unexpected keyword argument 'y'`, PyO3 sagt `scale_by() got an …`; bei `flip(.., z=1)` zählt pygame zuerst (`takes at most 3 arguments (4 given)`), PyO3 nennt zuerst den Namen. Der Satz ist nur zu treffen, indem **jede** Funktion `**kwargs` selbst nimmt — also an allen 600 oder an keiner. Wo ein Test darauf prüft, wird die Stelle einzeln nachgezogen. Seit Runde fünf steht dort wenigstens der **richtige** Funktionsname. |
| `draw.aaline` nennt 6, hier sind es 5 | pygame widerspricht sich selbst: mit siebzehn Argumenten `takes at most 6 arguments (17 given)`, mit sechs `takes at most 5 positional arguments (6 given)`. Die sechste Stelle ist das abgekündigte `blend`, das nur noch als Schlüsselwort durchgeht — und das nimmt rustgame genauso an. Eine falsche Zahl nachzuschreiben, um eine Probe grün zu bekommen, wäre das Gegenteil von messen. |
| `rotate_y_ip_rad`, `rotate_z_ip_rad` | **pygames Fehler, nicht unserer.** Beide drehen dort um die x-Achse; die neun anderen Varianten stimmen. Beim Zusammenlegen der alten und neuen Namen ist die Verdrahtung verrutscht — die Abkündigungswarnung sagt es mit. Einen falschen Drehsinn nachzubauen gäbe einem Programm stillschweigend Unsinn. |
| `argument 'x': ` vor dem Satz | PyO3 hängt den Namen des Parameters vor jeden Umwandlungsfehler, pygame nicht — an 51 Stellen. Die Namen sind seit Runde sieben pygames (`angle`, `length`, `type`); den Vorsatz loszuwerden heißt, jede dieser Funktionen auf `&Bound<PyAny>` umzubauen, und das zieht bei zwanzig Methoden eine Kette von Rückgabetypen nach sich. Zwei Stellen waren es wert und sind umgebaut. |
| `Line` ohne `__eq__`, `Cursor.type` schreibbar | **pygame ist mit sich uneinig.** `Line(1,2,3,4) == Line(1,2,3,4)` ist dort `False` und eine Strecke damit hashbar; sein eigener `Circle` macht es umgekehrt (Wertgleichheit, unhashbar). Hier folgt `Line` dem `Circle`. Und `Cursor.type`/`.data` sind bei pygame Fächer in einem `__dict__` — ohne jede Prüfung, `c.type = 77` geht dort; hier werden sie aus dem Zeiger abgeleitet und sind nur lesbar. |
| Wege ins Paket und die Fassung | `sysfont.pygame_main_file` zeigt bei pygame auf `pygame/__init__.py`, hier auf `rustgame_py.pyd` — die Hülle ist **eine Datei** und kein Ordner; `docs.PKG_DIR` dasselbe. `rev` ist hier `'rustgame'` und bei pygame `''`. Alle sechs sind in `Konstantenwerte` beim Namen ausgenommen, und jeder **andere** Wert von 1898 fällt weiterhin auf. |
| Wortlaut der Rekursionswächter | `Group.add('x')` läuft bei beiden in eine Endlosschleife (eine Zeichenkette ist eine Folge von Zeichenketten) und gibt bei beiden `RecursionError` — pygame mit CPythons `maximum recursion depth exceeded`, die Hülle mit PyO3s `Stack overflow (used 2912 kB) while calling a Python object`, dessen Zahl von Lauf zu Lauf wechselt. Der Satz stammt von der **Laufzeit**, nicht von pygame; die Art ist bei beiden dieselbe. |
| `__doc__` trägt deutsche Prosa | An einer Methode ist `__doc__` **nicht schreibbar** (`attribute '__doc__' of 'method_descriptor' objects is not writable`), und die Doku im Quelltext ist deutsch — so ist die Regel dieses Projekts. Es bleiben **530** Rufbare ohne Satz und **1063** mit deutscher Prosa; 160 Felder dazu. Module (48) und Klassen (94) tragen seit dieser Runde pygames Satz **wörtlich**, aus `gen_moduldoku.py` erzeugt. Die **Signatur**, die pygame in seine erste Doku-Zeile schreibt, zeigt `help()` auf beiden Seiten — hier aus einem echten `__text_signature__`. |
| `inspect.signature`: pygame hat meist keine | **662 von 2018** Rufbaren haben bei pygame keine Signatur (`ValueError: no signature found`), PyO3 erzeugt für jede eine. Eine Lücke nachzubauen ist in diesem Projekt dreimal als falsch gemessen worden. Wo beide eine haben, unterscheiden sich nur noch die **Namen** stellungsgebundener Parameter — pygames sind CPythons Platzhalter (`object`), unsere die Rust-Namen; rufen lässt sich beides gleich. Dazu `display.Info` (eine Fabrik mit `*args`, damit pygames genauer Satz `takes no arguments (1 given)` herauskommt) und zehn Sprite-Klassen, bei denen `inspect.signature(cls)` hier wirft, weil `__init__` ein Rust-Beschreiber ist. |
| vier Klassen lassen sich hier **ableiten** | `_camera.Camera`, `_render.Image`, `_render.Renderer`, `_render.Texture` sind bei pygame Cython-Typen ohne `Py_TPFLAGS_BASETYPE`; hier sind es Python-Klassen in den eingebetteten Modulen. Das zu verbieten hieße, ein `__init_subclass__` zu schreiben, das nichts tut als werfen — und es bricht kein Programm, das mit pygame läuft. In die andere Richtung waren es drei, und die sind behoben. |
| `Typliste halb angewandt` | `get([A, 2**31])` wirft hier wie dort `TypeError` — bei pygame sind die Ereignisse vom Typ `A` danach **weg**: es liest Glied für Glied und holt sie heraus, bevor es am zweiten scheitert. Bei `clear`, `set_blocked` und `set_allowed` ist das **nachgebaut** (und bei `peek`/`get_blocked` das Abbrechen beim ersten Treffer, was ändert, ob es überhaupt einen Fehler gibt). Nur bei `get` nicht: die herausgenommenen Ereignisse gibt pygame niemandem mehr — der Aufruf wirft ja —, und ein `get`, das die Schlange leert und nichts zurückgibt, wäre der eine Fall, in dem die Treue Daten vernichtet, die kein Programm mehr sehen kann. |
| pygame **stürzt** am halbfertigen Objekt ab | `font.Font`, `mask.Mask` und `_render.Texture`: `Klasse.__new__(Klasse)` und dann eine Eigenschaft lesen gibt `Fatal Python error: pygame_parachute: (pygame parachute) Segmentation Fault`. Hier antworten alle drei. Ein Absturz ist nichts, was man nachbaut — und `tools/halbfertig.py` fährt deshalb jede Klasse in einem eigenen Prozess. |
| Figuren und Zeiger stehen hier in **Rust** | pygames `sprite` und `cursors` sind reines Python. Ein halbfertiges Python-Objekt hat ein leeres `__dict__` und gibt auf **jede** Frage `AttributeError`; ein Rust-Objekt antwortet aus seinen Feldern (95 Stellen). Dieselbe Wurzel hat ein zweiter Unterschied: eine **ungebundene** Methode nur mit einem Schlüsselwort gerufen (`Klasse.methode(x=1)`) nennt bei einer C- oder Rust-Methode zuerst die fehlende Instanz, bei einer Python-Methode zuerst das Schlüsselwort — 157 Stellen hierhin, 21 dorthin. Zu treffen nur, indem `sprite` und `cursors` wieder in Python wandern. |
| `Event` und `Channel` ohne `__init__` | `Event.__new__(Event)` gibt bei pygame ein Ereignis mit `type` 0 und `dict` `None`, `Channel.__new__(Channel)` eine Spur mit `id` 0. Hier verlangen beide ihr Argument. Zu trennen wäre es wie bei `Color`, `Surface`, `Circle` und `Line` (ein `__init__` in Python setzt `tp_init`) — es bräuchte aber einen **erfundenen** Leerzustand im Kern: ein Ereignis ohne Wörterbuch und eine Spur ohne Mischer. |
| kaputte Klänge auf dem Weg von `music` | Vier Einträge. Dort meldet sich bei pygame der **Dekodierer selbst** (`stb_vorbis_open_rwops: VORBIS_invalid_setup`, `music_drflac: corrupt flac file (bad stream).`, `music_minimp3: …`, WavPack). Die Sätze hängen an den Fehlerpfaden von vier C-Bibliotheken; hier stehen andere Dekodierer. Bis Runde 19 waren es zehn: die `XMP:`-Sätze und AIFF sind seitdem nachgebaut. Auf dem Weg von `Sound` sagt pygame immer `Unrecognized audio format` — das war schon nachgebaut. |
| `Window` liest bei pygame **freigegebenen Speicher** | Nach `destroy()` geben `size` und `position` dort Zufallszahlen (gemessen `(999516864, 967804557)`, bei jedem Lauf andere); `id` und `opacity` geben `pygame.error: Invalid window`, `title` die leere Zeichenkette. Hier gibt es überall den Fehler — bis auf den Titel, der ist jetzt auch leer. Zufallszahlen sind nicht nachzubauen. |

#### Fünf Stellen, an denen rustgame **mehr** tut

Sie stehen nicht in der Tabelle darüber, weil sie keine Grenzen sind —
aber sie stehen hier, damit niemand sie für Schlamperei hält. Alle fünf
sind an Geräten, und bei allen fünf setzt pygame auf dieser Plattform
einen Aufruf nicht um oder nicht richtig:

| Aufruf | pygame (Windows) | rustgame |
|---|---|---|
| `camera.Camera.query_image()` | immer `False` | `True`, solange sie läuft |
| `camera.Camera.get_raw()` | immer `None` | die Bytes des Geräts |
| `camera.Camera.get_controls()[2]` | immer `-1` | der Reglerwert, sonst `-1` |
| `mouse.set_relative_mode(True)` | `No relative mode implementation available` | umgesetzt |
| `midi.Output(n)` nach einem `close()` | `Invalid device ID` bzw. `midi Output not open.` | öffnet |

Die ersten drei sind pygames MSMF-Stück: sein V4L2-Stück gibt bei
`get_raw()` die Bytes des Puffers heraus, die Lücke ist also keine
Absicht. Bei `query_image` wiegt sie schwer, denn pygames **eigenes**
Doku-Muster

```python
while not cam.query_image():
    pass
bild = cam.get_image()
```

hängt unter Windows für immer — gemessen war der Aufruf vierzigmal in
Folge `False`. Eine Lücke nachzubauen wäre hier die schlechtere Antwort,
und `-1` bei der Helligkeit heißt „weiß nicht", nicht „dunkel".

Die fünfte ist ein Fehler in pygame: nach `Output.close()` lässt sich in
derselben Sitzung **kein** MIDI-Ausgang mehr öffnen. Hier stand bis Runde
18 eine andere Zeile — `midi.get_count()` 2 gegen 1, weil `midir` den
`Microsoft MIDI Mapper` nicht zählt. Das war keine Stelle, an der
rustgame mehr tat, sondern eine Lücke, und **Gerät 0 war dadurch ein
anderes**. Der Mapper steht jetzt in PortMidis Reihenfolge vorn, und die
Probe `midi.Geraeteliste` vergleicht Anzahl und Inhalt.

Drei Grenzen von früher sind **weggefallen**, und jedes Mal war der
Vermerk das Problem, nicht die Umsetzung:

- **Versionsnummern von SDL_image, SDL_ttf und SDL_mixer** gaben `None`,
  mit der guten Begründung, rustgame binde keine der drei ein und eine
  erfundene Zahl schicke ein Programm lautlos in den falschen Zweig.
  pygames eigener `image_test.py` hat dann gezeigt, dass die Auskunft ein
  **Paar** ist: `get_extended()` und die Versionsnummer müssen zusammen
  ja oder zusammen nein sagen. Jetzt prüft der Kerntest genau diese
  Paarung — nicht die Zahl.
- **`music.get_pos` nach `stop()`+`load()`** ist keine Grenze mehr,
  sondern eine Probe.
- **In Runde 18 fielen drei weitere** — `smoothscale` (SSE2 Spur für Spur
  übertragen), die Glyphenbox (gehintete Umrisse und FreeTypes Rasterer)
  und `sysfont` (vier Fehler unter einem Vermerk, siehe dort).
- **`PixelArray` sperrte die Fläche nicht.** Die Begründung war richtig —
  PyO3 bildet `__init__` nicht auf `tp_init` ab, eine `#[pyclass]` sieht
  beim Bauen ihr eigenes Python-Objekt nicht —, nur der Schluss daraus
  war es nicht. Eine Unterklasse **in Python** hat den Schlitz von
  selbst; `pygame.PixelArray` ist jetzt dreißig Zeilen Python über der
  Rust-Klasse, und `s.get_locks()` gibt wieder das Feld selbst zurück.
  Eine Grenze, die an einer Bibliotheksfassung hängt, ist keine Grenze
  der Sache.

### Was die Prüfung insgesamt umfasst

| | |
|---|---:|
| Tests im Workspace | **668** |
| Tests in der Hülle (`rustgame-py`) | **12** — die liefen bis zuletzt **nie**, siehe unten |
| Tests aus pygames eigener Suite | **2364** grün von 2485 erreichbaren, **0 Abweichungen** |
| Testdateien, die gefahren werden | **alle 50** im Ordner, gelesen statt aufgezählt |
| Übersprungene Tests | **dieselben 121**, Name für Name — nicht nur dieselbe Zahl |
| Proben im Modulvergleich | **840**, davon 0 abweichend, **18** benannte Grenzen mit Abdruck (plus sieben Formgründe) |
| Zuweisungen an Felder | **1980** — 31 Instanzen, jedes Feld, elf Unsinnswerte |
| Konstanten**werte** samt Typ | **1898**, abgelesen statt genannt |
| Methoden mit falschem Argument | **431** |
| Klassen mit falschem Argument gebaut | **103** |
| Signaturformen (wo pygame eine hat) | **1354** |
| Docstrings von Modulen und Klassen, wörtlich | **142** |
| `dir()` gegen `dir()` | alle **60** Untermodule **und 65 Klassen**, **ohne** Ausnahmeliste |
| Referenzbilder | **123** |
| gewürfelte Folgen je Lauf | 600 (auf Wunsch 5000) |
| Pixelbytes im Programmvergleich | 0 von 76.800 abweichend |
| Fragen im Zustandsvergleich | **360** — 125 je Zustand, drei Zustände, eigene Prozesse |
| Kennzahlen des Pufferprotokolls | **722** — 77 Ausgeber, jede Kennzahl, die `memoryview` meldet |
| Fragen an halbfertige Objekte | **463** an 51 Klassen, je Klasse ein eigener Prozess |
| PyGameZ, Bild für Bild | **46 von 46** Spielen, je 300 Bilder, mit echtem Text — und zweimal gefahren |
| PyGameZ' eigene Audits auf rustgame | **14 von 14**, samt Zeitbudgets |
| Zeichenbilder von `font.render` | **11.984** normal und fett, **2.280** kursiv, **6.000** Sätze |
| kaputte Klang- und Bilddateien | **113** Fälle in vier Proben, zwei davon benannte Grenzen; WAV auf beiden Lesewegen |
| Vergleichsfälle der einzelnen Etappen | über **550.000** |

Die letzte Zeile ist keine Übertreibung: `transform` allein ist mit
78.093 Fällen und 28,6 Millionen Pixeln belegt, die Sprites mit 118.612,
die Kantenglättung mit 74.003, der Text mit 13.440, `FRect` mit 25.000,
`Rect` mit 10.000, `Color` mit 7.650 und die Vektoren mit 6.800 — jedes
Mal gegen **echtes pygame**, nicht gegen eine Erwartung.

### Der wiederkehrende Fehler: Vermerke veralten still

Die Abgleiche haben Fehler gefunden, die kein selbst ausgedachter Test
gefunden hätte: `Mask::overlap` suchte spaltenweise statt zeilenweise
(2564 von 2564 gegen 1412), `Rect::fit` rechnete mit dem falschen
Verhältnis, die Zeitgeber verschluckten verschlafene Wiederholungen. In
allen drei Fällen stand im Quelltext ein Kommentar, der die falsche
Variante mit „so macht es pygame" begründete — **ohne dass das je
nachgemessen worden war.**

Der zweite Fehler dieser Art ist mindestens so häufig: ein Vermerk, der
**einmal stimmte**. „Diese Kiste übersetzt gerade nicht." „`aaline`
fehlt, weil es im Kern fehlt." „pygame prüft `>=`, der Kern `>` — darum
eins abgezogen." Jeder dieser Sätze war richtig, als er geschrieben
wurde, und jeder hat danach eine Lücke gedeckt. Der letzte hat die Hülle
auf einer vollständig durchsichtigen Fläche die **ganze** Fläche melden
lassen, weil der Kern inzwischen selbst `>=` prüfte.

Der teuerste Fund dieser Art: `flags` kam in der Hülle als `u32` an.
`pygame.FULLSCREEN` ist `0x80000000` und wird darum als **negative** Zahl
geführt — `set_mode((64, 48), FULLSCREEN)` scheiterte also mit
`OverflowError`, der häufigste Vollbildaufruf überhaupt. Aufgefallen ist
das erst, als `display.mode_ok` dazukam und im Vergleich stolperte.
`set_mode` selbst hatte nie jemand mit `FULLSCREEN` geprüft.

Und die stillste Sorte: ein Wert, der **angenommen und weggeworfen**
wird. `transform.scale(s, größe, ziel)` trug den dritten Wert als
`let _ = dest_surface;`, damit der Aufruf nicht scheitert. Ein Programm,
das danach `ziel` benutzte, bekam eine unveränderte Fläche und keinerlei
Hinweis — schlimmer als ein Fehler, weil lautlos.

**In diesem Zug sind zwölf Vermerke gefallen**, und sie lehren dieselbe
Sache aus sechs Richtungen:

| Vermerk | was wirklich fehlte |
|---|---|
| „kein C-API, kein Puffer-Protokoll; bräuchte `unsafe`" | nicht `unsafe` im Arbeitsbereich, sondern `unsafe` **für genau eine Sache**: `puffer.rs` tut die Arbeit, fünf Dateien reichen die zwei PyO3-Schlitze durch. |
| „`unsafe` in genau einer Datei; die sechs anderen Kisten tragen `#![forbid(unsafe_code)]`" | **beides falsch, und das zweite schlimmer.** Es sind sechs Dateien (fünf davon reichen nur durch), und der Lint stand **nirgends** im Quelltext — die Aussage war wahr und völlig unbewacht. Jetzt steht er in allen sieben Kisten. |
| „`pixelcopy` braucht numpy; ohne Messung wird nichts gebaut" | nicht numpy. `pygame/tests/test_utils/buftools.py` und `arrinter.py` bauen Felder mit jeder Form und lagen im selben Ordner wie die Tests, die sie benutzen. |
| „`get_total_ram` bräuchte FFI, und wir haben kein `unsafe`" | eine Verwechslung. `unsafe` braucht **irgendwer** — die Kiste `sysinfo` tut es hinter einer sicheren Schnittstelle. |
| „`get_view`, `lock`, `get_locks` … kann es in Rust nicht geben" (in `EGAL` des Messgeräts) | gar nichts. Alle acht gab es längst; die Frage wurde nur nicht gestellt. |
| „`_sdl2.video.get_grabbed_window`: nur im Kern" | die Python-Seite zeigte auf `display.get_grabbed_window`, und das gibt es nicht. **Ein Zeiger auf eine leere Stelle ist schlimmer als kein Zeiger** — er sieht aus wie eine Umsetzung. |
| „`Mask`: ein `bool` je Stelle; der Unterschied ist Speicher, nicht Verhalten" | war wahr, bis `memoryview(mask)` ging. Seither **ist** die Anordnung im Speicher Teil der Schnittstelle, und `mask_test` rechnet mit ihr. Der Kern legt die Bits jetzt gepackt und spaltenweise ab, wie SDLs `bitmask_t`. |
| „`camera.Camera`: kein Gerätezugriff ohne FFI" | wieder `unsafe` verwechselt. `nokhwa` spricht mit Media Foundation, V4L2 und AVFoundation. |
| „`midi.Input`/`Output`: bräuchte PortMidi" | dasselbe mit `midir`. **Dreimal derselbe Satz, dreimal dieselbe Lücke in der Suche** — nicht in der Überlegung. |
| „`freetype.render`: Rasterkanten, Hinting gegen fontdue" | es waren keine Rasterkanten. Das Referenzbild `A_PyGameMono-8.png` ist aus dem **eingebetteten Bitmapschnitt** exportiert, und den gab es hier nicht. Zwei Tests hingen daran, und nur einer von beiden hatte „bitmap" im Namen. |
| „`ftfont.issue_font_alphablit`: Rasterkante an der Ecke" | **dasselbe Wort, dieselbe Bequemlichkeit.** `pygame.freetype` lud die Standardschrift um den Faktor 0,6875 zu klein — bei `ftfont` sogar doppelt. Das Bild war ein Drittel zu klein, nicht ein Pixel. |
| „`freetype.render_to_mono`: Rasterkante an der unteren linken Ecke" | **drei** Fehler hinter einem Wort: der Kasten eines Leerzeichens, das verschluckte Alpha der Schriftfarbe und `render_to` in einer statt zwei Stufen. |
| „die Testdateien werden aus dem Ordner gelesen statt aufgezählt" | sie wurden **abgetippt**. 47 Namen in der Liste, 50 Dateien im Ordner — und die drei fehlenden prüften genau zwei Namen, die rustgame nicht hatte. |
| „1085 von 1085 Namen (100 %)" | das Messgerät fragte jedes Untermodul und jede Klasse ab und **nie `dir(pygame)` selbst**. Fünf Namen fehlten: `print_debug_info`, `docs`, `surflock`, `SDL`, `rev`. |

Und einmal in die andere Richtung: ein **Test**, der eine Grenze
festschrieb. `was_ffi_braeuchte_sagt_das_auch` prüfte, dass
`get_total_ram()` einen Fehler gibt. Als die Funktion echt wurde, schlug
er an — nicht, weil etwas kaputt war, sondern weil er noch das alte
Versprechen hielt. **Ein Test, der eine Grenze pinnt, verteidigt sie gegen
die Messung**; er muss mit ihr verschwinden.

Und zuletzt die Sorte, die am meisten wehtut: **eine Probe, die eine
Stunde alt war.** `midi.Fehler` fragte nebenbei `midi._kanal_pruefen(16)`
ab — einen Namen, den **nur rustgame** hat, mit einem `hasattr` davor,
damit nichts scheitert. pygame lieferte vier Zeilen, rustgame sechs, und
das war die einzige Abweichung des Laufs. Keine davon lag an rustgame.

Es ist das fünfte Mal, dass das Messgerät selbst daneben lag, und das
erste Mal in einer Probe aus demselben Tag. Die Regel daraus: **eine Probe
darf nur nach Namen fragen, die pygame auch hat** — sonst vergleicht sie
rustgame mit rustgame.

Das sechste und siebte Mal kamen gleich danach, und sie gehören zusammen:
`pygame_tests.py` hatte eine **abgetippte** Dateiliste (47 von 50), und
`api_coverage.py` fragte **nie nach den Namen des Hauptmoduls**. Die
beiden Lücken deckten sich gegenseitig: `debug_test.py` prüft
`pygame.print_debug_info`, `docs_test.py` prüft `pygame.docs` — und beide
Namen hätte auch die API-Tabelle melden müssen. Zwei Messgeräte, eine
Lücke, und über ihr der Satz „die Testdateien werden aus dem Ordner
gelesen statt aufgezählt", der einfach nicht stimmte.

Jetzt liest `alle_dateien()` den Ordner, `REIHENFOLGE` gibt nur noch die
Lesefolge an, und `MODULE` hat einen Eintrag `""` für das Hauptmodul. Was
neu dazukommt, hängt hinten dran und fällt auf.

Und dabei fiel noch eine Falle auf, die **mich** dreimal erwischt hat,
nicht den Code: die Hülle läuft als Kopie
(`crates/rustgame-py/rustgame_py.pyd`) der gebauten `.dll`. Wer baut und
nicht kopiert, misst lautlos das alte Modul — `cargo build | grep -c
'^error'` gibt bei **null** Fehlern den Rückgabewert 1, und in einer
`&&`-Kette bleibt das `cp` danach aus. Dreimal zeigte die Suite dadurch
Abweichungen, die es nicht mehr gab.

**Die Regeln daraus:** wer auf einem Vermerk aufbaut, prüft ihn erst
nach. Eine Umrechnung zwischen zwei Schichten braucht auf **beiden**
Seiten einen Test, sonst überlebt sie die Schicht, die sie ausgleichen
sollte. Und ein Beiwert, den man nicht umsetzen kann, wird gemeldet und
nicht verschluckt.

---

## Lizenz

**LGPL-2.1** — anders als `rust/`, das MIT ist.

pygame-ce steht unter LGPL 2.1. Der schnellste und mit Abstand zuverlässigste
Weg zu pixelgleichem Verhalten war, die C-Quellen (`draw.c`, `transform.c`,
`alphablit.c`) als Vorlage zu nehmen und zu übertragen. Das erzeugt ein
abgeleitetes Werk, also gilt dieselbe Lizenz. Für ein Privatprojekt ist das
folgenlos.

**Geklärt ist damit aber nur pygames Teil.** Seit den Schriftrunden stecken
Übertragungen aus weiteren C-Bibliotheken darin, jede mit ihrer eigenen
Lizenz:

| Vorlage | Lizenz | steckt in |
|---|---|---|
| pygame-ce 2.5.7 (`draw.c`, `transform.c`, `alphablit.c`, `font.c`, …) | LGPL 2.1 | fast überall |
| SDL2, SDL_image, SDL_mixer, SDL_ttf | zlib | Formatwandler, Abtastratenwandler, Bild- und Tonformate, `SDL_ttf.c` |
| SDL_gfx in pygames Abzug (`SDL_gfxPrimitives.c`) | LGPL | `gfxdraw` |
| FreeType 2.11.1 (`ftgrays`, `ftraster`, `ftstroke`, `ttinterp`, `ttkern`, `fttrigon`) | FreeType-Lizenz (FTL) oder GPL 2 | `rustgame-font`, die rustgame-Zusätze in `vendor/skrifa` |
| skrifa 0.48.0 | MIT oder Apache 2.0 | `vendor/skrifa` |

Die FTL verlangt einen Vermerk in der Dokumentation, und hier steht er:
*Portions of this software are copyright © The FreeType Project
(www.freetype.org). All rights reserved.* Ob FTL-Teile und LGPL 2.1 so
zusammen weitergegeben werden dürfen, ist **vor einer Veröffentlichung**
zu klären; die Dateien in `tools/vorlage/` tragen die Köpfe ihrer
Herkunft.

Dass dieser Weg der richtige war, ist übrigens teuer gelernt: mehrere
Teilaufgaben wurden zuerst aus Messreihen zu rekonstruieren versucht und
blieben an widersprüchlichen Daten hängen. Mit der Vorlage daneben waren sie
in einem Bruchteil der Zeit pixelgenau.
