# vendor/ — fremde Kisten mit rustgame-Zusatz

## skrifa 0.48.0

Unverändert aus crates.io (`skrifa-0.48.0`, MIT/Apache-2.0), dazu die
folgenden Zusätze, jeder mit `rustgame:` im Kommentar. pygame-ce 2.5.7
bringt FreeType **2.11.1** mit; skrifa folgt einem neueren FreeType. Wo
die beiden auseinandergehen, gilt hier 2.11.1 — gemessen an pygames
eigener `freetype.dll` (ctypes-Orakel), nicht gelesen.

| Datei | Zusatz | Seit |
|---|---|---|
| `src/outline/glyf/hint/instance.rs`, `src/outline/hint.rs` | `scan_zustand()`: `GS.scan_control`/`scan_type` nach `prep` — bei abgeschaltetem Hinting und `INSTCTRL` Bit 2 die Vorgabe | Runde 19, 20 |
| `src/outline/hint.rs` | `vorschub_roh()`/`vorschub_roh_leeren()`: der Vorschub **vor** dem Runden | Runde 19 |
| `src/outline/hint.rs`, `glyf/hint/graphics.rs`, `glyf/hint/engine/*.rs`, `glyf/hint/math.rs`, `glyf/mod.rs` | **Zwei Größenachsen** (`HintingInstance::new_gestreckt`): FreeTypes gestreckter Interpreter — `Current_Ratio` mit `FT_Vector_Length` (CORDIC aus `fttrigon.c`, gegen die DLL geprüft), `Read_CVT_Stretched`/`Write_CVT_Stretched`/`Move_CVT_Stretched`, `Current_Ppem_Stretched` für `MPPEM` und `DELTA*`, zwei Skalen in `Ins_MD`/`Ins_MDRP`/`Ins_IP`, Punkte mit `x_scale` und `y_scale` getrennt | Runde 20 |
| `src/outline/hint.rs`, `src/outline/mod.rs`, `glyf/mod.rs` | Von `prep` **abgeschaltetes** Hinting: FreeType lädt dann ohne Anweisungen, aber mit den Skalen des Laders (`hinted_metrics`, bei `head.flags` Bit 3 gerundet) — `draw_abgeschaltet`, `mit_lade_skalen` | Runde 20 |
| `src/outline/hint.rs` | `roh_umriss()`: der Umriss als FreeTypes Punktliste, mit der Art jedes Punktes **nach** dem Hinting (`FLIPPT`/`FLIPRGON` ändern sie — Courier New fett) | Runde 20 |

**Warum:** rustgame zeichnet Text pixelgleich mit pygame. Dafür braucht es
Zustände und Werte, die skrifa intern berechnet und nicht herausgibt
(Dropout-Modus, ungerundeter Vorschub, Punktarten nach dem Hinting), und
zwei Fälle, die skrifa nicht kennt (zwei Achsen, abgeschaltetes Hinting
mit FreeTypes Skalen).

**Geprüft und bewusst nicht geändert:** `GETINFO` Selektor 5
(„Graustufen“). FreeType 2.11 setzt `exc->grayscale` im v40-Interpreter
auf `!subpixel_hinting_lean && …` — bei `TARGET_NORMAL` also nie. skrifa
lässt das Bit weg und hat damit recht; ein Versuch, es zu setzen, hat
Courier New bei 23,5 sofort verdorben.

**Wieder loswerden:** sobald skrifa diese Fälle selbst kann, die zwei
`[patch.crates-io]`-Zeilen (in `rustgame/Cargo.toml` und
`crates/rustgame-py/Cargo.toml`) und diesen Ordner streichen.
