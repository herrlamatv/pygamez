//! transform.rs
//! ============
//! Gegenstueck zu `pygame.transform`: skalieren, spiegeln, drehen,
//! weichzeichnen, faerben, ausschneiden.
//!
//! ## Was hier stimmt - und was nur ungefaehr
//!
//! Nichts davon ist geraten. Die Rechenwege sind die von pygame-ce 2.5.7
//! (`src_c/transform.c`, `src_c/rotozoom.c`, `src_c/scale2x.c`), und jede
//! Funktion wurde anschliessend gegen echtes pygame nachgemessen: erst
//! 615 Faelle fuer die Skalierer, dann 78 093 weitere Faelle mit
//! 28,6 Millionen Pixeln fuer alles Uebrige - zufaellige Groessen,
//! Muster, Winkel, Faktoren und Randfaelle, jeder Pixel als rohes
//! 32-Bit-Wort verglichen.
//!
//! | Funktion | Genauigkeit gegen pygame-ce 2.5.7 | gemessen |
//! |---|---|---|
//! | [`scale`], [`scale_by`] | **pixelgenau** | 120 Faelle, 112 228 Bytes, 0 Abweichungen |
//! | [`flip`] | **pixelgenau** (reines Umsortieren) | 30 Faelle, 12 020 Bytes, 0 |
//! | [`rotate`] | **pixelgenau**, auch bei krummen Winkeln | 240 Faelle, 158 208 Bytes, 0 |
//! | [`scale2x`], [`grayscale`], [`average_color`] | **pixelgenau** | 60 Faelle, 0 |
//! | [`smoothscale`], [`smoothscale_by`] | **pixelgenau auf beiden Rechenwegen** (`SSE2` und `GENERIC`) | siehe [`smoothscale`] |
//! | [`box_blur`], [`gaussian_blur`] | **pixelgenau** | je 5 951 Faelle, je 680 218 Pixel, 0 |
//! | [`hsl`] | **pixelgenau** (samt `f32`-Rundung) | 6 308 Faelle, 628 360 Pixel, 0 |
//! | [`chop`], [`pixelate`] | **pixelgenau**, auch bei negativen Rechtecken | 8 692 Faelle, 721 688 Pixel, 0 |
//! | [`invert`], [`laplacian`] | **pixelgenau** in allen sichtbaren Kanaelen | je 604 Faelle, je 68 432 Pixel, 0 |
//! | [`solid_overlay`] | **pixelgenau**, samt dem leeren Sonderfall | 3 972 Faelle, 402 936 Pixel, 0 |
//! | [`average_surfaces`] | **pixelgenau** | 1 119 Faelle, 128 392 Pixel, 0 |
//! | [`threshold`] | **pixelgenau**, Rueckgabewert in allen Faellen gleich | 9 424 Faelle, 933 608 Pixel, 0 |
//! | [`rotozoom`] | **pixelgenau**, Groesse in allen Faellen gleich | 35 468 Faelle, 24,3 Mio. Pixel, 0 |
//! | [`get_smoothscale_backend`], [`set_smoothscale_backend`] | meldet **`SSE2`** wie pygame auf jedem x86-Rechner, schaltet auf `GENERIC` um |  |
//!
//! Zu `smoothscale`: pygame bringt **zwei** Umsetzungen desselben Filters
//! mit (`GENERIC` in C, `SSE2` in Vektorbefehlen), und die weichen
//! voneinander ab - in 74 von 98 gemessenen Faellen. Beide stehen hier,
//! Befehl fuer Befehl, und beide treffen ihr Vorbild **Byte fuer Byte**.
//!
//! Hier stand bis Runde 18, naeher komme an dieser Stelle "niemand heran,
//! auch pygame selbst nicht", und gerechnet wurde nur `GENERIC`. Das
//! verwechselte zwei Fragen: dass pygames zwei Wege **voneinander**
//! abweichen, sagt nichts darueber, ob man **einen** davon treffen kann.
//! `SSE2` ist reine 16-Bit-Ganzzahlarithmetik - deterministisch und genauso
//! nachzurechnen wie C. Gekostet hat der Vermerk **jedes Bild** von
//! PyGameZ: dessen Hintergruende entstehen aus `smoothscale`, und ein
//! Programm, das nichts umstellt, rechnet bei pygame `SSE2`.
//!
//! ## Die drei Rechenwege in Kurzform
//!
//! * **`scale`** ist Naechster-Nachbar mit 16.16-Festkomma und halbem
//!   Pixelversatz (SDLs `SDL_SoftStretchNearest`):
//!   `schritt = (quelle << 16) / ziel`, Startwert `schritt / 2`.
//!   Gemessen 4 -> 7: die Quellindizes `[0, 0, 1, 1, 2, 3, 3]`.
//! * **`smoothscale`** laeuft in zwei Durchgaengen (erst X, dann Y). Beim
//!   Verkleinern wird die Flaeche gemittelt (Kastenfilter mit
//!   16.16-Zaehler), beim Vergroessern linear zwischen zwei Nachbarn
//!   gemischt - mit pygames eigenem Versatz `x * (quelle - 1) / ziel`, der
//!   den letzten Quellpixel nie ganz erreicht.
//! * **`rotate`** prueft `winkel % 90 == 0` und dreht dann durch blosses
//!   Umsortieren. Sonst wird die Zielflaeche groesser, und jedes Zielpixel
//!   holt sich ueber eine 16.16-Festkommagerade seinen Quellpixel. Was
//!   daneben liegt, bekommt die **Hintergrundfarbe**: die Schluesselfarbe,
//!   sonst das Pixel links oben (bei Flaechen mit Alphakanal durchsichtig
//!   gemacht). Das ist pygames bekannte Eigenart, keine Nachlaessigkeit.
//!
//! ## Weichzeichnen
//!
//! In der Spielesammlung wird `smoothscale` auch zweckentfremdet: erst
//! verkleinern, dann wieder vergroessern ergibt einen Weichzeichner. Das
//! funktioniert hier genauso, weil beide Richtungen dieselben Filter
//! benutzen wie das Vorbild. Fuer diesen Zweck gibt es zusaetzlich
//! [`weichzeichnen`].
//!
//! ## Vier bewusste Abweichungen
//!
//! * Flaechen **ohne** `SRCALPHA` bekommen im Ergebnis durchgehend
//!   Alpha 255. pygame rechnet dort auf einem ungenutzten vierten Byte
//!   herum, das niemand je zu sehen bekommt (`Amask == 0`), und legt je
//!   nach Funktion etwas anderes hinein: `grayscale` den Grauwert
//!   (`00411eff` wird `43434343`), `invert` das umgedrehte Byte
//!   (`00010203` wird `fffefdfc`). Hier bleibt dort eine Null stehen -
//!   das ist die Konvention der ganzen Kiste (siehe
//!   [`Surface::abbilden`](crate::surface::Surface::abbilden)), und
//!   sichtbar wird der Unterschied nie. Nachgemessen: ueber 28,6
//!   Millionen Pixel weicht **nur** dieses eine Byte ab, und nur bei
//!   `invert` und `grayscale`.
//! * Ist die **Quelle** leer (Breite oder Hoehe 0), kommt eine leere bzw.
//!   unveraenderte Flaeche zurueck. pygame liest an dieser Stelle teils
//!   ueber den Puffer hinaus.
//! * [`box_blur`] mit `radius == min(breite, hoehe)` liest in C eine
//!   Zeile und eine Spalte ueber den Puffer hinaus - pygames Klemme
//!   greift erst bei **groesser**. Gemessen kommt dort Speichermuell
//!   heraus. Hier werden die Zugriffe stattdessen auf die letzte Zeile
//!   bzw. Spalte geklemmt; alle anderen Radien stimmen ueberein.
//! * [`average_surfaces`] verlangt, dass keine Flaeche kleiner ist als die
//!   erste. pygame liest sonst ueber deren Puffer hinaus (gemessen:
//!   `[3x3, 2x2]` liefert ein 3x3-Ergebnis mit Speichermuell).
//!
//! Ein Unterschied, der **nicht** hierher gehoert, sondern in `surface.rs`
//! sitzt: pygame meldet fuer eine frisch angelegte `SRCALPHA`-Flaeche
//! `get_alpha() == 255`, rustgame `None`. Die Funktionen hier reichen das
//! nur durch.

use std::sync::atomic::{AtomicU8, Ordering};

use crate::color::{Color, IntoColor};
use crate::rect::Rect;
use crate::surface::Surface;

// ---------------------------------------------------------------------------
//  Kleine Helfer
// ---------------------------------------------------------------------------

/// Ein Bild als Zeilen aus RGBA-Vierergruppen.
///
/// pygames Filter arbeiten auf Bytes in Speicherreihenfolge und behandeln
/// alle vier Kanaele gleich - deshalb ist die Reihenfolge hier egal,
/// solange sie durchgehalten wird.
type Zeilen = Vec<Vec<[u8; 4]>>;

/// Die Pixel einer Flaeche, zeilenweise.
fn lies_flach(s: &Surface) -> Vec<[u8; 4]> {
    let roh = s.to_rgba_bytes();
    (0..roh.len() / 4)
        .map(|i| [roh[i * 4], roh[i * 4 + 1], roh[i * 4 + 2], roh[i * 4 + 3]])
        .collect()
}

/// Die Pixel einer Flaeche, wie `average_color` sie sieht.
///
/// Bei 24 und 32 Bit genau [`lies_flach`]; bei einem schmaleren Format
/// gehen die unteren Bits verloren und werden **nicht** wieder
/// aufgefuellt. Siehe [`average_color`], warum.
fn mittelbare_kanaele(s: &Surface) -> Vec<[u8; 4]> {
    let format = s.get_format();
    let (lr, lg, lb, _) = format.losses();
    if s.get_palette().is_none() && lr == 0 && lg == 0 && lb == 0 {
        return lies_flach(s);
    }
    let (sr, sg, sb, _) = format.shifts();
    let (mr, mg, mb, _) = format.masks;
    lies_flach(s)
        .iter()
        .map(|p| {
            // Zurueck in die Bits des Formats und von dort wieder heraus -
            // so, wie pygame sie im Pixel findet.
            let v = format.packen(
                ((p[3] as u32) << 24)
                    | ((p[0] as u32) << 16)
                    | ((p[1] as u32) << 8)
                    | p[2] as u32,
            );
            [
                ((((v & mr) >> sr) << lr) & 0xFF) as u8,
                ((((v & mg) >> sg) << lg) & 0xFF) as u8,
                ((((v & mb) >> sb) << lb) & 0xFF) as u8,
                p[3],
            ]
        })
        .collect()
}

/// Die Pixel einer Flaeche als Zeilen.
fn lies_zeilen(s: &Surface) -> Zeilen {
    let (w, h) = s.get_size();
    let flach = lies_flach(s);
    (0..h as usize)
        .map(|y| flach[y * w as usize..(y + 1) * w as usize].to_vec())
        .collect()
}

/// Eine leere Flaeche mit den Eigenschaften der Vorlage.
///
/// Entspricht pygames `newsurf_fromsurf`: Pixelformat, Flaechen-Alpha,
/// Mischmodus und Schluesselfarbe werden uebernommen, der Inhalt nicht.
/// Genau dafuer gibt es [`Surface::wie`] - von Hand nachgezogen ginge der
/// Mischmodus verloren, denn `set_alpha(None)` setzt ihn auf "nicht
/// mischen".
///
/// **Der eine Fall, in dem `newsurf_fromsurf` mehr tut als kopieren:** den
/// Mischmodus der neuen Flaeche bestimmt nicht der der Vorlage, sondern
/// `_PgSurface_SrcAlpha` - und das entscheidet bei einer Flaeche **ohne**
/// Alphakanal am Flaechen-Alpha:
///
/// ```c
/// if (SDL_ISPIXELFORMAT_ALPHA(...)) { ... mode == SDL_BLENDMODE_BLEND -> 1 }
/// else { PG_GetSurfaceAlphaMod(surf, &color); if (color != SDL_ALPHA_OPAQUE) return 1; }
/// return 0;
/// ```
///
/// Eine Flaeche ohne Alphakanal mit `set_alpha(255)` meldet also `0`, und
/// die neue Flaeche bekommt `SDL_BLENDMODE_NONE` - ihr `get_alpha()` ist
/// danach `None`, obwohl die Vorlage 255 meldet. Gemessen an echtem
/// pygame ueber alle Funktionen, die hier durchlaufen (`invert`, `hsl`,
/// `chop`, `pixelate`, beide Weichzeichner, `laplacian`,
/// `solid_overlay`):
///
/// ```text
/// b = Surface((4,4)); b.set_alpha(255)
/// b.get_alpha()                      -> 255
/// transform.invert(b).get_alpha()    -> None
/// ```
fn neu_wie(vorlage: &Surface, size: (u32, u32)) -> Surface {
    let mut s = Surface::wie(vorlage, size);
    if !vorlage.has_alpha() && vorlage.get_alpha() == Some(255) {
        s.set_alpha(None);
    }
    s
}

/// Die Pixel einer Flaeche als **rohe** Formatwerte (ARGB8888 je `u32`).
///
/// Anders als [`lies_flach`] wird das vierte Byte hier **nicht** maskiert.
/// Das ist fuer alles noetig, was pygame byteweise rechnet: die
/// Weichzeichner, `rotozoom` und `solid_overlay` fassen den ungenutzten
/// vierten Kanal einer Flaeche ohne Alphakanal genauso an wie die drei
/// anderen.
fn lies_roh(s: &Surface) -> Vec<u32> {
    let (w, h) = s.get_size();
    let mut v = Vec::with_capacity((w as usize) * (h as usize));
    for y in 0..h as i32 {
        for x in 0..w as i32 {
            v.push(s.get_at_mapped((x, y)));
        }
    }
    v
}

/// Rohe Formatwerte in eine neue Flaeche schreiben.
///
/// Ohne Alphakanal wird das ungenutzte vierte Byte **geleert** - das ist
/// die Konvention dieser Kiste (siehe [`Surface::abbilden`]). pygames
/// SIMD-Wege legen dort je nach Funktion Muell ab: `grayscale` den
/// Grauwert (gemessen: `00411eff` wird `43434343`), `invert` das
/// umgedrehte Byte (`00010203` wird `fffefdfc`). Sichtbar ist davon
/// nichts, `get_at` maskiert es - aber `scale2x` vergleicht ganze Woerter,
/// deshalb bleibt es hier bei der Null.
fn schreibe_roh(vorlage: &Surface, size: (u32, u32), daten: &[u32]) -> Surface {
    let mut ziel = neu_wie(vorlage, size);
    let maske = if vorlage.has_alpha() {
        0xFFFF_FFFFu32
    } else {
        0x00FF_FFFFu32
    };
    let (w, h) = (size.0 as usize, size.1 as usize);
    // Schmale Formate und Palettenflaechen koennen den Rohwert nicht
    // halten: dort muss er erst auf das Darstellbare gebracht werden. Bei
    // 24 und 32 Bit ist `einrasten` die Identitaet, also bleibt der
    // geprueste Weg Wort fuer Wort derselbe.
    let roh_genuegt = ziel.get_palette().is_none() && ziel.get_format().verlustfrei();
    let eingerastet: Option<Vec<u32>> = if roh_genuegt {
        None
    } else {
        Some(daten.iter().map(|v| ziel.einrasten(*v)).collect())
    };
    let quelle: &[u32] = eingerastet.as_deref().unwrap_or(daten);
    ziel.mit_pixeln(|puffer, zeilenbreite, (ox, oy)| {
        for y in 0..h {
            for x in 0..w {
                puffer[(oy + y) * zeilenbreite + ox + x] = quelle[y * w + x] & maske;
            }
        }
    });
    ziel
}

/// Traegt fertige Pixel in eine neue Flaeche ein.
///
/// Ohne `SRCALPHA` wird das Alpha auf 255 gezwungen - siehe Modulkopf.
fn schreibe(vorlage: &Surface, size: (u32, u32), daten: &[[u8; 4]]) -> Surface {
    let mut ziel = neu_wie(vorlage, size);
    let hat_alpha = vorlage.has_alpha();
    if hat_alpha {
        ziel.setze_alle(daten);
    } else {
        let deckend: Vec<[u8; 4]> = daten.iter().map(|p| [p[0], p[1], p[2], 255]).collect();
        ziel.setze_alle(&deckend);
    }
    ziel
}

/// Zeilen wieder flach legen.
fn flach_aus(z: &Zeilen) -> Vec<[u8; 4]> {
    z.iter().flat_map(|r| r.iter().copied()).collect()
}

// ---------------------------------------------------------------------------
//  scale - Naechster Nachbar
// ---------------------------------------------------------------------------

/// Naechster-Nachbar-Skalierung (`pygame.transform.scale`).
///
/// Schnell, hart, ohne Mischen: jedes Zielpixel bekommt genau einen
/// Quellpixel. Das ist SDLs `SDL_SoftStretchNearest` - 16.16-Festkomma mit
/// halbem Pixelversatz, damit die Abtastung in der Mitte des Quellpixels
/// sitzt und nicht an seiner Kante:
///
/// ```text
/// schritt = (quelle << 16) / ziel
/// index   = (schritt / 2 + i * schritt) >> 16
/// ```
///
/// Gemessen an echtem pygame (4 -> 7): `[0, 0, 1, 1, 2, 3, 3]`. Das ist
/// **pixelgenau** nachgebaut, ueber 384 Groessenpaare geprueft.
pub fn scale(s: &Surface, size: (u32, u32)) -> Surface {
    let (sw, sh) = s.get_size();
    let (dw, dh) = size;
    // pygame laesst leere Flaechen unangetastet - hier ebenso.
    if dw == 0 || dh == 0 || sw == 0 || sh == 0 {
        return neu_wie(s, size);
    }

    let quelle = lies_flach(s);
    let schritt_x = ((sw as i64) << 16) / dw as i64;
    let schritt_y = ((sh as i64) << 16) / dh as i64;

    let mut ziel = Vec::with_capacity((dw * dh) as usize);
    let mut pos_y = schritt_y / 2;
    for _ in 0..dh {
        let qy = ((pos_y >> 16) as u32).min(sh - 1) as usize;
        pos_y += schritt_y;
        let mut pos_x = schritt_x / 2;
        for _ in 0..dw {
            let qx = ((pos_x >> 16) as u32).min(sw - 1) as usize;
            pos_x += schritt_x;
            ziel.push(quelle[qy * sw as usize + qx]);
        }
    }
    schreibe(s, size, &ziel)
}

/// Skalierung um einen Faktor (`pygame.transform.scale_by`).
///
/// Die Zielgroesse wird wie in pygame **abgeschnitten**, nicht gerundet:
/// 5 mal 2,4 ergibt 12.
pub fn scale_by(s: &Surface, fx: f32, fy: f32) -> Surface {
    scale(s, faktor_groesse(s, fx, fy))
}

/// Zielgroesse aus einem Faktor - abschneiden wie pygames `(int)`-Umwandlung.
fn faktor_groesse(s: &Surface, fx: f32, fy: f32) -> (u32, u32) {
    let w = (s.get_width() as f32 * fx) as i64;
    let h = (s.get_height() as f32 * fy) as i64;
    (w.max(0) as u32, h.max(0) as u32)
}

// ---------------------------------------------------------------------------
//  smoothscale - Kastenfilter beim Verkleinern, linear beim Vergroessern
// ---------------------------------------------------------------------------

/// Bilineare Skalierung (`pygame.transform.smoothscale`).
///
/// Zwei Durchgaenge, erst waagerecht, dann senkrecht - genau wie pygames
/// `scalesmooth`. Wird kleiner skaliert, mittelt ein Kastenfilter ueber die
/// zusammenfallenden Quellpixel; wird groesser skaliert, wird zwischen zwei
/// Nachbarn gemischt.
///
/// Gerechnet wird auf dem Weg, den [`set_smoothscale_backend`] gewaehlt
/// hat - vorgegeben ist `SSE2`, wie bei pygame auf jedem x86-Rechner.
///
/// **Genauigkeit:** auf beiden Wegen pixelgleich mit pygame. Gemessen an
/// 98 Faellen (sieben Quell- mal sieben Zielgroessen von 1x1 bis 640x480,
/// mit und ohne Alphakanal, Zufallspixel): `SSE2` gegen `SSE2` und
/// `GENERIC` gegen `GENERIC` je **0** Abweichungen; die zwei Wege
/// untereinander weichen in 74 der 98 Faelle ab.
///
/// Gemessen an echtem pygame (`GENERIC`), 4 -> 8 Pixel breit, Quelle
/// `[schwarz, rot, gruen, blau]`:
/// `[(0,0,0), (95,0,0), (191,0,0), (223,31,0), (127,127,0), (31,223,0),
///   (0,191,63), (0,95,159)]`.
pub fn smoothscale(s: &Surface, size: (u32, u32)) -> Surface {
    // Der Weg wird **einmal** gelesen: ein Umschalten aus einem anderen
    // Faden darf nicht mitten in einer Flaeche die Rechnung wechseln.
    smoothscale_mit(s, size, rechenweg())
}

/// [`smoothscale`] auf einem ausdruecklich genannten Rechenweg - ohne den
/// prozessweiten Schalter anzufassen (die Kerntests laufen parallel).
fn smoothscale_mit(s: &Surface, size: (u32, u32), weg: Rechenweg) -> Surface {
    let (sw, sh) = s.get_size();
    let (dw, dh) = size;
    if dw == 0 || dh == 0 {
        return neu_wie(s, size);
    }
    // pygame behandelt die gleiche Groesse als reine Kopie - kein Filter,
    // kein Rundungsfehler. Daran haengt mehr, als man denkt: nur deshalb
    // ist "smoothscale auf die eigene Groesse" verlustfrei.
    if (sw, sh) == (dw, dh) {
        return s.copy();
    }
    if sw == 0 || sh == 0 {
        return neu_wie(s, size);
    }

    // Erst die Breite, dann die Hoehe - wie in pygames scalesmooth.
    let mut bild = achse(&lies_zeilen(s), sw as usize, dw as usize, weg);
    if dh != sh {
        bild = umklappen(&achse(&umklappen(&bild), sh as usize, dh as usize, weg));
    }
    schreibe(s, size, &flach_aus(&bild))
}

/// Skalierung um einen Faktor (`pygame.transform.smoothscale_by`).
pub fn smoothscale_by(s: &Surface, fx: f32, fy: f32) -> Surface {
    smoothscale(s, faktor_groesse(s, fx, fy))
}

/// Weichzeichnen ueber zweimaliges `smoothscale`.
///
/// Kein eigener Filter, sondern der Kniff, mit dem die Spielesammlung ihre
/// Glas- und Scheinoptik baut: erst verkleinern, dann wieder auf die alte
/// Groesse aufblasen. `staerke` ist der Verkleinerungsfaktor - 4 heisst
/// "auf ein Viertel und zurueck". Unter 1 passiert nichts.
pub fn weichzeichnen(s: &Surface, staerke: f32) -> Surface {
    let (w, h) = s.get_size();
    if staerke <= 1.0 || w == 0 || h == 0 {
        return s.copy();
    }
    let klein = (
        ((w as f32 / staerke) as u32).max(1),
        ((h as f32 / staerke) as u32).max(1),
    );
    smoothscale(&smoothscale(s, klein), (w, h))
}

/// Ein Durchgang laengs der Zeilen: verkleinern, vergroessern oder nichts.
fn achse(quelle: &Zeilen, sn: usize, dn: usize, weg: Rechenweg) -> Zeilen {
    match (dn.cmp(&sn), weg) {
        (std::cmp::Ordering::Less, Rechenweg::Generic) => schrumpfe(quelle, sn, dn),
        (std::cmp::Ordering::Less, Rechenweg::Sse2) => schrumpfe_sse2(quelle, sn, dn),
        (std::cmp::Ordering::Greater, Rechenweg::Generic) => dehne(quelle, sn, dn),
        (std::cmp::Ordering::Greater, Rechenweg::Sse2) => dehne_sse2(quelle, sn, dn),
        (std::cmp::Ordering::Equal, _) => quelle.clone(),
    }
}

/// Zeilen und Spalten tauschen.
///
/// pygame hat fuer die Senkrechte eigene Funktionen (`filter_shrink_Y`,
/// `filter_expand_Y`). Sie rechnen Byte fuer Byte dasselbe wie die
/// waagerechten - auf **beiden** Wegen: die SSE2-Fassungen arbeiten
/// zwei Zeilen bzw. zwei Pixel auf einmal, aber jede Rechnung bleibt in
/// ihrer 16-Bit-Spur, nichts mischt zwischen Kanaelen oder Zeilen.
/// Deshalb steht jeder Filter hier nur einmal.
fn umklappen(bild: &Zeilen) -> Zeilen {
    if bild.is_empty() {
        return Vec::new();
    }
    let breite = bild[0].len();
    (0..breite)
        .map(|x| bild.iter().map(|zeile| zeile[x]).collect())
        .collect()
}

/// Verkleinern: Flaechenmittelung mit 16.16-Zaehler.
///
/// Wort fuer Wort pygames `filter_shrink_X_ONLYC`. Der Zaehler laeuft
/// ueber die Quellpixel; sobald ein Zielpixel voll ist, wird es
/// ausgeschrieben und der Rest des angebrochenen Quellpixels ins naechste
/// hinuebergerettet.
///
/// Der Zwischenspeicher ist wie im Vorbild nur 16 Bit breit. Bei mehr als
/// 257 Quellpixeln je Zielpixel laeuft er ueber - dieser Fehler ist
/// bewusst mit uebernommen, damit das Ergebnis auch dort dasselbe ist.
fn schrumpfe(quelle: &Zeilen, sn: usize, dn: usize) -> Zeilen {
    let spanne = 0x10000i64 * sn as i64 / dn as i64;
    let kehrwert = 0x1_0000_0000i64 / spanne;

    quelle
        .iter()
        .map(|zeile| {
            let mut ziel = vec![[0u8; 4]; dn];
            let mut summe = [0u16; 4];
            let mut zaehler = spanne;
            let mut k = 0usize;

            for p in zeile.iter().take(sn) {
                if zaehler > 0x10000 {
                    // Dieser Quellpixel gehoert ganz zum laufenden Zielpixel.
                    for (s, v) in summe.iter_mut().zip(p.iter()) {
                        *s = s.wrapping_add(*v as u16);
                    }
                    zaehler -= 0x10000;
                } else {
                    // Hier endet das Zielpixel mitten im Quellpixel.
                    let rest = 0x10000 - zaehler;
                    if k < dn {
                        for (z, (s, v)) in
                            ziel[k].iter_mut().zip(summe.iter().zip(p.iter()))
                        {
                            let anteil = (*v as i64 * zaehler) >> 16;
                            *z = (((*s as i64 + anteil) * kehrwert) >> 16) as u8;
                        }
                        k += 1;
                    }
                    // Was vom Quellpixel uebrig ist, beginnt das naechste.
                    for (s, v) in summe.iter_mut().zip(p.iter()) {
                        *s = ((*v as i64 * rest) >> 16) as u16;
                    }
                    zaehler = spanne - rest;
                }
            }
            ziel
        })
        .collect()
}

/// Vergroessern: lineares Mischen zweier Nachbarn.
///
/// Wort fuer Wort pygames `filter_expand_X_ONLYC`. Der Versatz
/// `x * (quelle - 1) / ziel` ist pygames eigener - er erreicht den letzten
/// Quellpixel nie ganz. Die Gewichte sind 16.16-Festkomma (`0x10000`);
/// mit `0x100` waere jedes zweite Pixel eine Stufe daneben - nachgemessen.
fn dehne(quelle: &Zeilen, sn: usize, dn: usize) -> Zeilen {
    let mut index = Vec::with_capacity(dn);
    let mut gewicht1 = Vec::with_capacity(dn);
    for x in 0..dn {
        let t = x * (sn - 1);
        index.push(t / dn);
        gewicht1.push((0x10000i64 * (t % dn) as i64 / dn as i64) as i32);
    }

    quelle
        .iter()
        .map(|zeile| {
            (0..dn)
                .map(|x| {
                    let i0 = index[x];
                    // Bei i0 == sn-1 ist das Gewicht immer 0; pygame liest
                    // dort ueber die Zeile hinaus, das Ergebnis ist aber
                    // dasselbe. Wir bleiben im Puffer.
                    let i1 = (i0 + 1).min(sn - 1);
                    let m1 = gewicht1[x];
                    let m0 = 0x10000 - m1;
                    let mut aus = [0u8; 4];
                    for (a, (v0, v1)) in aus
                        .iter_mut()
                        .zip(zeile[i0].iter().zip(zeile[i1].iter()))
                    {
                        *a = (((*v0 as i32 * m0) + (*v1 as i32 * m1)) >> 16) as u8;
                    }
                    aus
                })
                .collect()
        })
        .collect()
}

// ---------------------------------------------------------------------------
//  smoothscale - der SSE2-Weg
// ---------------------------------------------------------------------------
//
// Uebertragen aus pygame-ce 2.5.7, `src_c/simd_transform_sse2.c`
// (`filter_shrink_X_SSE2`, `filter_expand_X_SSE2` und ihre Y-Zwillinge).
// Die Vektorbefehle werden **einzeln** nachgerechnet, jeder auf einer
// 16-Bit-Spur - SIMD heisst nur, dass acht solche Spuren gleichzeitig
// laufen, und keine sieht die andere. Was diesen Weg von `GENERIC`
// unterscheidet, sind drei Dinge, und alle drei muessen mit:
//
// * **14 statt 16 Bit Festkomma** beim Verkleinern (`0x4000` statt
//   `0x10000`), und die Teilgewichte entstehen ueber `_mm_mulhi_epu16` -
//   die oberen 16 Bit eines Produkts, also **abgeschnitten**.
// * **8 statt 16 Bit Gewichte** beim Vergroessern (`0x100`).
// * **`_mm_packus_epi16` liest mit Vorzeichen.** Ein Zwischenwert ueber
//   `0x7FFF` gilt als negativ und wird **0**, nicht 255. Bei mehr als 257
//   Quellpixeln je Zielpixel laeuft der 16-Bit-Zwischenspeicher ueber,
//   genau wie im Vorbild - und dann entscheidet dieses Bit.

/// `_mm_mulhi_epu16`: die oberen 16 Bit eines vorzeichenlosen 16x16-Produkts.
fn mulhi_epu16(a: u16, b: u16) -> u16 {
    ((a as u32 * b as u32) >> 16) as u16
}

/// `_mm_packus_epi16`: die Spur **mit Vorzeichen** gelesen und auf
/// `0..=255` gesaettigt.
fn packus_epi16(v: u16) -> u8 {
    (v as i16).clamp(0, 255) as u8
}

/// Verkleinern auf dem SSE2-Weg (`filter_shrink_X_SSE2`).
///
/// Dieselbe Flaechenmittelung wie [`schrumpfe`], aber in 14-Bit-Festkomma:
/// `spanne = 0x4000 * sn / dn`, der Kehrwert `0x40000000 / spanne` auf
/// **16 Bit** gestutzt (`(Uint16)` im Vorbild), jeder Quellwert vor dem
/// Gewichten um zwei Bit nach links geschoben (`_mm_slli_epi16(src, 2)`),
/// damit `mulhi` mit einem 14-Bit-Gewicht wieder auf Pixeleinheiten kommt.
fn schrumpfe_sse2(quelle: &Zeilen, sn: usize, dn: usize) -> Zeilen {
    // `int` im Vorbild; `0x04000 * srcwidth` passt bis 131071 Pixel.
    let spanne = (0x4000i64 * sn as i64 / dn as i64) as i32;
    let kehrwert = (0x4000_0000i64 / spanne as i64) as u16;

    quelle
        .iter()
        .map(|zeile| {
            let mut ziel = vec![[0u8; 4]; dn];
            let mut summe = [0u16; 4];
            let mut zaehler = spanne;
            let mut k = 0usize;

            for p in zeile.iter().take(sn) {
                if zaehler > 0x4000 {
                    // `_mm_add_epi16`: der Zwischenspeicher laeuft ueber.
                    for (s, v) in summe.iter_mut().zip(p.iter()) {
                        *s = s.wrapping_add(*v as u16);
                    }
                    zaehler -= 0x4000;
                } else {
                    // `zaehler` liegt hier in (0, 0x4000], `rest` in
                    // [0, 0x4000) - beide passen in eine 16-Bit-Spur.
                    let rest = 0x4000 - zaehler;
                    let mut aus = [0u8; 4];
                    for (c, a) in aus.iter_mut().enumerate() {
                        let v = (p[c] as u16) << 2;
                        let d = mulhi_epu16(v, zaehler as u16).wrapping_add(summe[c]);
                        summe[c] = mulhi_epu16(v, rest as u16);
                        *a = packus_epi16(mulhi_epu16(d, kehrwert));
                    }
                    if k < dn {
                        ziel[k] = aus;
                    }
                    k += 1;
                    zaehler = spanne - rest;
                }
            }
            ziel
        })
        .collect()
}

/// Vergroessern auf dem SSE2-Weg (`filter_expand_X_SSE2`).
///
/// Dieselbe Mischung zweier Nachbarn wie [`dehne`], aber mit
/// **8-Bit-Gewichten**: `xm1 = 0x100 * ((x * (sn - 1)) % dn) / dn`, das
/// Ergebnis `(p0 * xm0 + p1 * xm1) >> 8`. Die Produkte entstehen mit
/// `_mm_mullo_epi16`; weil `xm0 + xm1 = 256` ist, bleibt die Summe unter
/// `255 * 256` und laeuft nie ueber.
fn dehne_sse2(quelle: &Zeilen, sn: usize, dn: usize) -> Zeilen {
    let mut index = Vec::with_capacity(dn);
    let mut gewicht1 = Vec::with_capacity(dn);
    for x in 0..dn {
        let t = x * (sn - 1);
        index.push(t / dn);
        gewicht1.push((0x100 * (t % dn) / dn) as u32);
    }

    quelle
        .iter()
        .map(|zeile| {
            (0..dn)
                .map(|x| {
                    let i0 = index[x];
                    // Wie bei `dehne`: am letzten Pixel ist das Gewicht 0,
                    // pygame liest daneben, wir bleiben im Puffer.
                    let i1 = (i0 + 1).min(sn - 1);
                    let m1 = gewicht1[x];
                    let m0 = 0x100 - m1;
                    let mut aus = [0u8; 4];
                    for (a, (v0, v1)) in aus
                        .iter_mut()
                        .zip(zeile[i0].iter().zip(zeile[i1].iter()))
                    {
                        *a = ((*v0 as u32 * m0 + *v1 as u32 * m1) >> 8) as u8;
                    }
                    aus
                })
                .collect()
        })
        .collect()
}

// ---------------------------------------------------------------------------
//  flip
// ---------------------------------------------------------------------------

/// Spiegeln (`pygame.transform.flip`).
///
/// Reines Umsortieren, also **pixelgenau** und verlustfrei. `x` spiegelt
/// waagerecht (links/rechts), `y` senkrecht (oben/unten).
pub fn flip(s: &Surface, x: bool, y: bool) -> Surface {
    let (w, h) = s.get_size();
    if w == 0 || h == 0 {
        return s.copy();
    }
    let quelle = lies_flach(s);
    let mut ziel = Vec::with_capacity((w * h) as usize);
    for zy in 0..h {
        let qy = if y { h - 1 - zy } else { zy };
        for zx in 0..w {
            let qx = if x { w - 1 - zx } else { zx };
            ziel.push(quelle[(qy * w + qx) as usize]);
        }
    }
    schreibe(s, (w, h), &ziel)
}

// ---------------------------------------------------------------------------
//  rotate
// ---------------------------------------------------------------------------

/// Drehen gegen den Uhrzeigersinn (`pygame.transform.rotate`).
///
/// Bei Vielfachen von 90 Grad wird nur umsortiert - **verlustfrei und
/// pixelgenau**, die Flaeche behaelt ihre Pixel und tauscht nur Breite und
/// Hoehe. Bei jedem anderen Winkel wird die Zielflaeche groesser (das
/// umschliessende Rechteck des gedrehten Bildes), und jeder Zielpixel holt
/// sich ueber eine 16.16-Festkommagerade seinen Quellpixel; was daneben
/// faellt, bekommt die Hintergrundfarbe.
///
/// Die neue Groesse berechnet pygame aus den gedrehten Eckpunkten und
/// **schneidet ab**: 12 mal 8 bei 45 Grad ergibt 14 mal 14 (nicht 15).
pub fn rotate(s: &Surface, winkel: f32) -> Surface {
    let (w, h) = s.get_size();
    if w < 1 || h < 1 {
        return s.copy();
    }
    // pygame prueft genau so: fmod(winkel, 90) == 0.
    if (winkel as f64) % 90.0 == 0.0 {
        return drehe_viertel(s, (winkel as i32) / 90);
    }

    let bogen = winkel as f64 * 0.017_453_292_519_943_29;
    let (sinus, kosinus) = (bogen.sin(), bogen.cos());

    // Das umschliessende Rechteck, Ecke fuer Ecke - wie im Vorbild.
    let (x, y) = (w as f64, h as f64);
    let (cx, cy, sx, sy) = (kosinus * x, kosinus * y, sinus * x, sinus * y);
    let groesster = |a: f64, b: f64| a.max(b);
    let nxmax = groesster(
        groesster(groesster((cx + sy).abs(), (cx - sy).abs()), (-cx + sy).abs()),
        (-cx - sy).abs(),
    ) as i32;
    let nymax = groesster(
        groesster(groesster((sx + cy).abs(), (sx - cy).abs()), (-sx + cy).abs()),
        (-sx - cy).abs(),
    ) as i32;
    let size = (nxmax.max(0) as u32, nymax.max(0) as u32);
    if size.0 == 0 || size.1 == 0 {
        return neu_wie(s, size);
    }

    let hintergrund = hintergrundfarbe(s);
    let quelle = lies_flach(s);
    let (dw, dh) = (size.0 as i32, size.1 as i32);

    // Ab hier alles in 16.16-Festkomma, genau wie pygames rotate().
    let mitte_y = dh / 2;
    let xd = (w as i32 - dw) << 15;
    let yd = (h as i32 - dh) << 15;
    let isin = (sinus * 65536.0) as i32;
    let icos = (kosinus * 65536.0) as i32;
    let ax = (dw << 15) - (kosinus * (((dw - 1) as i64) << 15) as f64) as i32;
    let ay = (dh << 15) - (sinus * (((dw - 1) as i64) << 15) as f64) as i32;
    let xmax = ((w as i32) << 16) - 1;
    let ymax = ((h as i32) << 16) - 1;

    let mut ziel = Vec::with_capacity((dw * dh) as usize);
    for zy in 0..dh {
        let mut dx = ax
            .wrapping_add(isin.wrapping_mul(mitte_y - zy))
            .wrapping_add(xd);
        let mut dy = ay
            .wrapping_sub(icos.wrapping_mul(mitte_y - zy))
            .wrapping_add(yd);
        for _ in 0..dw {
            if dx < 0 || dy < 0 || dx > xmax || dy > ymax {
                ziel.push(hintergrund);
            } else {
                let qx = (dx >> 16) as usize;
                let qy = (dy >> 16) as usize;
                ziel.push(quelle[qy * w as usize + qx]);
            }
            dx = dx.wrapping_add(icos);
            dy = dy.wrapping_add(isin);
        }
    }
    schreibe(s, size, &ziel)
}

/// Die Farbe, mit der pygame die Ecken einer schraeg gedrehten Flaeche
/// fuellt.
///
/// Gibt es eine Schluesselfarbe, ist sie es. Sonst nimmt pygame **das Pixel
/// links oben** - und loescht dessen Alpha, wenn die Flaeche einen
/// Alphakanal hat. Wer sich schon gewundert hat, warum gedrehte Bilder in
/// pygame manchmal einen farbigen Rahmen bekommen: hier steht der Grund.
fn hintergrundfarbe(s: &Surface) -> [u8; 4] {
    if let Some(k) = s.get_colorkey() {
        return [k.r, k.g, k.b, k.a];
    }
    let p = s.get_at((0, 0));
    [p.r, p.g, p.b, if s.has_alpha() { 0 } else { p.a }]
}

/// Drehen um ein Vielfaches von 90 Grad - verlustfrei.
fn drehe_viertel(s: &Surface, viertel: i32) -> Surface {
    let mut n = viertel % 4;
    if n < 0 {
        n += 4;
    }
    let (w, h) = s.get_size();
    let size = if n % 2 == 0 { (w, h) } else { (h, w) };
    if w == 0 || h == 0 {
        return neu_wie(s, size);
    }

    let quelle = lies_flach(s);
    let hole = |x: u32, y: u32| quelle[(y * w + x) as usize];
    let mut ziel = Vec::with_capacity((w * h) as usize);
    for y in 0..size.1 {
        for x in 0..size.0 {
            ziel.push(match n {
                1 => hole(w - 1 - y, x),
                2 => hole(w - 1 - x, h - 1 - y),
                3 => hole(y, h - 1 - x),
                _ => hole(x, y),
            });
        }
    }
    schreibe(s, size, &ziel)
}

// ---------------------------------------------------------------------------
//  Zugaben
// ---------------------------------------------------------------------------

/// Verdoppeln nach AdvanceMAME Scale2x (`pygame.transform.scale2x`).
///
/// Kein Filter, sondern ein Musterverfahren: aus einem Pixel werden vier,
/// und wo die vier Nachbarn es hergeben, werden Schraegen geglaettet statt
/// Bloecke gebaut. Fuer Pixelgrafik deutlich besser als [`scale`] mit
/// Faktor 2 - und **pixelgenau** wie das Vorbild.
pub fn scale2x(s: &Surface) -> Surface {
    let (w, h) = s.get_size();
    let size = (w * 2, h * 2);
    if w == 0 || h == 0 {
        return neu_wie(s, size);
    }
    // **Roh, nicht ueber `lies_flach`.**
    //
    // `scale2x` ist die einzige Funktion hier, die Pixel nicht rechnet,
    // sondern **vergleicht** - und `scale2x.c` vergleicht ganze 32-Bit-
    // Woerter, samt des vierten Bytes. Auf einer Flaeche ohne Alphakanal
    // ist dieses Byte unsichtbarer Zustand: meist null, nach einem Blit
    // mit Flaechen-Alpha `0xFF`, nach einer `BLEND_RGBA_ADD`- oder
    // `-MAX`-Mischart der gerechnete Wert. Wer hier ueber `to_rgba_bytes`
    // geht, sieht ueberall 255 und haelt Pixel fuer gleich, die es in
    // pygame nicht sind.
    let q: Vec<u32> = {
        let mut roh = s.clone();
        let mut v = Vec::with_capacity((w * h) as usize);
        roh.mit_pixeln(|daten, breite, (ox, oy)| {
            for y in 0..h as usize {
                let zeile = (oy + y) * breite + ox;
                v.extend_from_slice(&daten[zeile..zeile + w as usize]);
            }
        });
        v
    };
    let hole = |x: u32, y: u32| q[(y * w + x) as usize];

    let mut ziel = vec![0u32; (size.0 * size.1) as usize];
    for y in 0..h {
        for x in 0..w {
            let e = hole(x, y);
            let b = hole(x, y.saturating_sub(1));
            let d = hole(x.saturating_sub(1), y);
            let f = hole((x + 1).min(w - 1), y);
            let hh = hole(x, (y + 1).min(h - 1));
            let (e0, e1, e2, e3) = if b != hh && d != f {
                (
                    if d == b { d } else { e },
                    if b == f { f } else { e },
                    if d == hh { d } else { e },
                    if hh == f { f } else { e },
                )
            } else {
                (e, e, e, e)
            };
            let i = (y * 2 * size.0 + x * 2) as usize;
            ziel[i] = e0;
            ziel[i + 1] = e1;
            ziel[i + size.0 as usize] = e2;
            ziel[i + size.0 as usize + 1] = e3;
        }
    }
    // Ebenso roh zurueckschreiben - das vierte Byte gehoert dazu.
    let mut aus = neu_wie(s, size);
    aus.mit_pixeln(|daten, breite, (ox, oy)| {
        for y in 0..size.1 as usize {
            let zeile = (oy + y) * breite + ox;
            daten[zeile..zeile + size.0 as usize]
                .copy_from_slice(&ziel[y * size.0 as usize..(y + 1) * size.0 as usize]);
        }
    });
    aus
}

/// Graustufen (`pygame.transform.grayscale`).
///
/// pygame nimmt die Helligkeitsformel von OpenCV, aber in Festkomma -
/// damit die Vektorfassung dasselbe herausbekommt wie die einfache:
///
/// ```text
/// grau = ((76*r + 255) >> 8) + ((150*g + 255) >> 8) + ((29*b + 255) >> 8)
/// ```
///
/// Das Alpha bleibt stehen. Gemessen: (200, 100, 50) wird 125.
pub fn grayscale(s: &Surface) -> Surface {
    let (w, h) = s.get_size();
    let grau: Vec<[u8; 4]> = lies_flach(s)
        .iter()
        .map(|p| {
            let g = (((76u32 * p[0] as u32 + 255) >> 8)
                + ((150u32 * p[1] as u32 + 255) >> 8)
                + ((29u32 * p[2] as u32 + 255) >> 8)) as u8;
            [g, g, g, p[3]]
        })
        .collect();
    schreibe(s, (w, h), &grau)
}

/// Durchschnittsfarbe (`pygame.transform.average_color`).
///
/// `bereich` schraenkt auf ein Rechteck ein (`None` = die ganze Flaeche).
/// Mit `mit_alpha` werden die Farben nach ihrem Alpha gewichtet - fast
/// durchsichtige Pixel zaehlen dann kaum mit.
///
/// Wie in pygame hat eine Flaeche **ohne** `SRCALPHA` keinen Alphakanal,
/// den man mitteln koennte; das Ergebnis hat dort Alpha 0 (gemessen).
///
/// ## Warum `average_color` bei 16 Bit **nicht** dasselbe sieht wie `get_at`
///
/// pygame holt die Kanaele hier von Hand aus dem Pixel:
/// `((pixel & mask) >> shift) << loss`. Das ist **nicht** SDLs
/// `SDL_GetRGB`, das die fehlenden Bits auffuellt
/// (`(v * 255) / ((1 << (8 - loss)) - 1)`). Der Unterschied ist gemessen
/// und macht bis zu sieben Stufen aus:
///
/// ```text
/// Surface((4,4), depth=16), gefuellt mit    get_at        average_color
///   (255,   0,   0)                       (255,  0,  0)   (248,  0,  0)
///   ( 74,  76,  74)                       ( 74, 76, 74)   ( 72, 76, 72)
/// Surface((4,4), depth=8)  (Palette)      (  0,  0,  0)   (  0,  0,  0)
/// ```
///
/// `transform_test.test_grayscale` haengt genau daran: es faerbt eine
/// 16-Bit-Flaeche grau und erwartet von `average_color` `(72, 76, 72)`,
/// waehrend `get_at` dort `(74, 76, 74)` liest. Bei 24 und 32 Bit ist
/// `loss` null und beide Wege fallen zusammen - deshalb ist das nie
/// aufgefallen.
///
/// Auf einer Palettenflaeche sind alle Masken null, und damit ist die
/// Durchschnittsfarbe **schwarz** - auch das ist pygames Verhalten, nicht
/// ein Versehen hier.
pub fn average_color(s: &Surface, bereich: Option<Rect>, mit_alpha: bool) -> Color {
    let ganz = s.get_rect();
    let r = match bereich {
        None => ganz,
        Some(b) => b.normalized().clip(ganz),
    };
    if r.is_empty() {
        return Color::rgba(0, 0, 0, 0);
    }
    let (w, _) = s.get_size();
    let pixel = mittelbare_kanaele(s);
    let hat_alpha = s.has_alpha();

    let (mut rt, mut gt, mut bt, mut at) = (0u64, 0u64, 0u64, 0u64);
    for y in r.y..r.y + r.h {
        for x in r.x..r.x + r.w {
            let p = pixel[(y as u32 * w + x as u32) as usize];
            let a = if hat_alpha { p[3] as u64 } else { 0 };
            at += a;
            if mit_alpha {
                rt += p[0] as u64 * a;
                gt += p[1] as u64 * a;
                bt += p[2] as u64 * a;
            } else {
                rt += p[0] as u64;
                gt += p[1] as u64;
                bt += p[2] as u64;
            }
        }
    }
    let anzahl = (r.w as u64) * (r.h as u64);
    // Bei Alphagewichtung teilt pygame durch die Alphasumme - ist die 0,
    // faellt es auf die Pixelzahl zurueck.
    let teiler = if mit_alpha && at != 0 { at } else { anzahl };
    Color::rgba(
        (rt / teiler) as u8,
        (gt / teiler) as u8,
        (bt / teiler) as u8,
        (at / anzahl) as u8,
    )
}

// ---------------------------------------------------------------------------
//  invert
// ---------------------------------------------------------------------------

/// Farben umkehren (`pygame.transform.invert`).
///
/// `~r, ~g, ~b` - das Alpha bleibt unangetastet. Wort fuer Wort pygames
/// `invert_non_simd`:
///
/// ```c
/// PG_GetRGBA(pixel, src_format, src_palette, &r, &g, &b, &a);
/// Uint32 new_pixel = PG_MapRGBA(newsurf_format, newsurf_palette, ~r, ~g, ~b, a);
/// ```
///
/// Gemessen an echtem pygame: `(1,2,3)` wird `(254,253,252)`,
/// `(128,128,128,7)` wird `(127,127,127,7)`.
///
/// **Zum vierten Byte:** pygame nimmt hier auf dieser Maschine den
/// AVX2-Weg, und der dreht das ungenutzte Byte einer Flaeche **ohne**
/// Alphakanal mit um (Rohspeicher gemessen: `00010203` wird `fffefdfc`).
/// Hier bleibt es bei der Null - siehe `schreibe_roh`.
pub fn invert(s: &Surface) -> Surface {
    let (w, h) = s.get_size();
    let daten: Vec<u32> = lies_roh(s)
        .iter()
        .map(|v| {
            let [b, g, r, a] = v.to_le_bytes();
            u32::from_le_bytes([!b, !g, !r, a])
        })
        .collect();
    schreibe_roh(s, (w, h), &daten)
}

// ---------------------------------------------------------------------------
//  box_blur / gaussian_blur
// ---------------------------------------------------------------------------

/// Kasten-Weichzeichner (`pygame.transform.box_blur`).
///
/// Zwei getrennte Durchgaenge mit gleitender Summe: erst senkrecht, dann
/// waagerecht, beides auf **Bytes in Speicherreihenfolge**. Der vierte
/// Kanal wird mitgerechnet - bei einer Flaeche ohne Alphakanal ist das das
/// ungenutzte Byte, das dort ueberall null ist und null bleibt.
///
/// `repeat_edge_pixels` (pygames Vorgabe: `true`) verlaengert das Bild am
/// Rand mit seinem eigenen Randpixel; `false` mittelt am Rand ueber
/// weniger Pixel, teilt aber trotzdem durch `2*radius+1` - der Rand wird
/// also dunkler. Das ist pygames Verhalten, kein Versehen.
///
/// Gemessen an echtem pygame (4x3, Pixel `(x*60, y*80, (x+y)*40)`,
/// `radius=1`): die ersten drei Pixel werden `(20,26,26) (60,26,53)
/// (120,26,93)`.
pub fn box_blur(
    s: &Surface,
    radius: i32,
    repeat_edge_pixels: bool,
) -> Result<Surface, &'static str> {
    weichzeichner(s, radius, repeat_edge_pixels, false)
}

/// Gauss-Weichzeichner (`pygame.transform.gaussian_blur`).
///
/// `radius` ist in pygame das **Sigma** der Glockenkurve; der Kern reicht
/// `2*sigma` weit. Gerechnet wird in `f32`, genau wie im Vorbild
/// (`expf`, nicht `exp`).
///
/// **Der Sonderfall, den keine Messreihe erklaert:** bei `sigma == 0`
/// steht in der C-Quelle `expf(-powf(0,2) / (2*powf(0,2)))`, also
/// `expf(-0.0/0.0)` - das ist `NaN`. Jeder Ausgabewert wird dann
/// `(Uint8)NaN`, und das ist auf x86 **0**. pygame liefert dort also eine
/// vollstaendig schwarze, durchsichtige Flaeche. Nachgemessen:
/// `gaussian_blur(flaeche, 0)` ergibt lauter `00000000`. Und das trifft
/// mehr Faelle als man denkt, denn `radius` wird vorher geklemmt: auf
/// einer nur ein Pixel hohen Flaeche wird aus **jedem** Radius groesser 1
/// eine Null (gemessen an 1x5 mit `radius=3`).
///
/// Gemessen (4x3 wie oben, `radius=1`): `(21,28,28) (63,28,56)
/// (116,28,91)`.
pub fn gaussian_blur(
    s: &Surface,
    radius: i32,
    repeat_edge_pixels: bool,
) -> Result<Surface, &'static str> {
    weichzeichner(s, radius, repeat_edge_pixels, true)
}

/// Der gemeinsame Vorbau beider Weichzeichner - pygames `blur()`.
fn weichzeichner(
    s: &Surface,
    radius: i32,
    repeat: bool,
    gauss: bool,
) -> Result<Surface, &'static str> {
    if radius < 0 {
        return Err("The radius should not be less than zero.");
    }
    let (w, h) = s.get_size();
    if w == 0 || h == 0 {
        // pygame steigt hier aus, bevor ueberhaupt gerechnet wird.
        return Ok(neu_wie(s, (w, h)));
    }
    // pygames Klemme - und sie greift erst bei **groesser**, nicht bei
    // gleich. Genau deshalb liest `box_blur(flaeche_3x3, 3)` in C eine
    // Zeile ueber den Puffer hinaus; gemessen kommt dort Speichermuell
    // heraus (das vierte Byte war 0x25, 0x22, 0x1f ...). Hier werden die
    // Zugriffe stattdessen auf die letzte Zeile bzw. Spalte geklemmt -
    // bewusste Abweichung, weil "undefiniert" nichts ist, das man
    // nachbauen kann.
    let mut r = radius;
    let kleinste = w.min(h) as i32;
    if r > kleinste {
        r = kleinste - 1;
    }
    let bytes = if gauss {
        gauss_bytes(s, r, repeat)
    } else {
        kasten_bytes(s, r, repeat)
    };
    let daten: Vec<u32> = bytes
        .as_chunks::<4>().0.iter()
        .map(|c| u32::from_le_bytes([c[0], c[1], c[2], c[3]]))
        .collect();
    Ok(schreibe_roh(s, (w, h), &daten))
}

/// Die Pixel einer Flaeche als Bytes in Speicherreihenfolge (B, G, R, A).
///
/// So sieht pygame sie: seine Filter laufen ueber `Uint8 *pixels` und
/// kennen die Bedeutung der einzelnen Bytes gar nicht.
fn lies_bytes(s: &Surface) -> Vec<u8> {
    lies_roh(s).iter().flat_map(|v| v.to_le_bytes()).collect()
}

/// Wort fuer Wort pygames `box_blur()`.
fn kasten_bytes(s: &Surface, r: i32, repeat: bool) -> Vec<u8> {
    let (w, h) = (s.get_width() as usize, s.get_height() as usize);
    let nb = 4usize;
    let pitch = w * nb;
    let q = lies_bytes(s);
    let ru = r as usize;
    let teiler = (r * 2 + 1) as u32;
    // Geklemmte Zugriffe - siehe `weichzeichner`.
    let zeile = |y: usize| y.min(h - 1) * pitch;
    let spalte = |x: usize| x.min(w - 1) * nb;

    let mut sum_v = vec![0u32; pitch];
    for y in 0..=ru {
        for (i, z) in sum_v.iter_mut().enumerate() {
            *z = z.wrapping_add(q[zeile(y) + i] as u32);
        }
    }
    if repeat {
        for (i, z) in sum_v.iter_mut().enumerate() {
            *z = z.wrapping_add((q[i] as u32).wrapping_mul(r as u32));
        }
    }

    let mut buf = vec![0u32; pitch];
    let mut aus = vec![0u8; h * pitch];
    for y in 0..h {
        for i in 0..pitch {
            buf[i] = sum_v[i] / teiler;
            // Die senkrechte Summe weiterschieben.
            if y as i32 - r >= 0 {
                sum_v[i] = sum_v[i].wrapping_sub(q[(y - ru) * pitch + i] as u32);
            } else if repeat {
                sum_v[i] = sum_v[i].wrapping_sub(q[i] as u32);
            }
            if y as i32 + r + 1 < h as i32 {
                sum_v[i] = sum_v[i].wrapping_add(q[(y + ru + 1) * pitch + i] as u32);
            } else if repeat {
                sum_v[i] = sum_v[i].wrapping_add(q[(h - 1) * pitch + i] as u32);
            }
        }

        let mut sum_h = [0u32; 4];
        for x in 0..=ru {
            for (c, sh) in sum_h.iter_mut().enumerate() {
                *sh = sh.wrapping_add(buf[spalte(x) + c]);
            }
        }
        if repeat {
            for (c, sh) in sum_h.iter_mut().enumerate() {
                *sh = sh.wrapping_add(buf[c].wrapping_mul(r as u32));
            }
        }
        for x in 0..w {
            for (c, sh) in sum_h.iter_mut().enumerate() {
                aus[y * pitch + nb * x + c] = (*sh / teiler) as u8;
                // Die waagerechte Summe weiterschieben.
                if x as i32 - r >= 0 {
                    *sh = sh.wrapping_sub(buf[(x - ru) * nb + c]);
                } else if repeat {
                    *sh = sh.wrapping_sub(buf[c]);
                }
                if x as i32 + r + 1 < w as i32 {
                    *sh = sh.wrapping_add(buf[(x + ru + 1) * nb + c]);
                } else if repeat {
                    *sh = sh.wrapping_add(buf[(w - 1) * nb + c]);
                }
            }
        }
    }
    aus
}

/// Wort fuer Wort pygames `gaussian_blur()`.
fn gauss_bytes(s: &Surface, sigma: i32, repeat: bool) -> Vec<u8> {
    let (w, h) = (s.get_width() as usize, s.get_height() as usize);
    let nb = 4usize;
    let pitch = w * nb;
    let mut aus = vec![0u8; h * pitch];
    // Der `NaN`-Fall aus der C-Quelle - siehe [`gaussian_blur`].
    if sigma == 0 {
        return aus;
    }
    let q = lies_bytes(s);
    let kr = sigma * 2;

    // Die Glockenkurve - in `f32`, und die Summe wird waehrend des
    // Aufbaus mitgezaehlt, nicht danach. Die Reihenfolge steht so in der
    // Quelle und entscheidet ueber das letzte Bit.
    let mut lut = vec![0f32; (kr + 1) as usize];
    let mut lut_sum = 0.0f32;
    for (i, l) in lut.iter_mut().enumerate() {
        *l = (-(i as f32).powf(2.0) / (2.0 * (sigma as f32).powf(2.0))).exp();
        lut_sum += *l * 2.0;
    }
    lut_sum -= lut[0];
    for l in lut.iter_mut() {
        *l /= lut_sum;
    }

    let mut buf = vec![0f32; pitch];
    let mut buf2 = vec![0f32; pitch];
    for y in 0..h as i32 {
        for j in -kr..=kr {
            let g = lut[j.unsigned_abs() as usize];
            for (i, b) in buf.iter_mut().enumerate() {
                if y + j >= 0 && y + j < h as i32 {
                    *b += q[pitch * (y + j) as usize + i] as f32 * g;
                } else if repeat {
                    let z = if y + j < 0 { 0 } else { h - 1 };
                    *b += q[pitch * z + i] as f32 * g;
                }
            }
        }
        for x in 0..w as i32 {
            for j in -kr..=kr {
                let g = lut[j.unsigned_abs() as usize];
                for c in 0..nb {
                    let ziel = nb * x as usize + c;
                    if x + j >= 0 && x + j < w as i32 {
                        buf2[ziel] += buf[nb * (x + j) as usize + c] * g;
                    } else if repeat {
                        let sp = if x + j < 0 { 0 } else { w - 1 };
                        buf2[ziel] += buf[nb * sp + c] * g;
                    }
                }
            }
        }
        for i in 0..pitch {
            aus[pitch * y as usize + i] = buf2[i] as u8;
            buf[i] = 0.0;
            buf2[i] = 0.0;
        }
    }
    aus
}

// ---------------------------------------------------------------------------
//  laplacian
// ---------------------------------------------------------------------------

/// Kantenerkennung (`pygame.transform.laplacian`).
///
/// Der Kern ist der uebliche:
///
/// ```text
/// -1 -1 -1
/// -1  8 -1
/// -1 -1 -1
/// ```
///
/// **Drei Eigenarten aus der C-Quelle, die keine Messung an einem
/// Farbverlauf zeigt** (dort ist das Ergebnis ueberall null, weil der
/// Laplace-Operator lineare Verlaeufe ausloescht):
///
/// 1. Fehlende Nachbarn am Rand werden nicht ausgelassen, sondern durch
///    `LAPLACIAN_NUM = 0xFFFFFFFF` ersetzt - also durch **weiss**. Ein
///    Rand wirkt damit wie eine weisse Umrandung.
/// 2. Die Mitte wird **zweimal** gezaehlt: `sample[4]` ist beim Aufsummieren
///    noch `0` (und `0` ergibt entschluesselt `(0,0,0)` mit Alpha 255 ohne
///    bzw. 0 mit Alphakanal), erst danach wird der echte Mittelpixel
///    geholt. Die Summe laeuft also ueber neun Werte, von denen einer
///    immer schwarz ist.
/// 3. `sample[0]`, `sample[2]`, `sample[6]` und `sample[8]` werden am
///    linken und rechten Rand **gar nicht gesetzt** - dort steht noch der
///    Wert aus einem frueheren Durchlauf. Beim allerersten Pixel ist das
///    uninitialisierter Stapelspeicher; gemessen verhaelt er sich wie 0
///    (Beleg: eine weisse 3x3-Flaeche ergibt bei (0,0) genau `255`, und
///    `255*8 - 1785 - x = 255` geht nur mit `x = 0` auf).
///
/// Ein Beleg fuer die Punkte 1 und 3, gemessen an echtem pygame: eine
/// 5x5-Flaeche, schwarz, mit `(255,200,100)` in Spalte 0 und
/// `(250,210,90)` in Spalte 4, ergibt in Spalte 4 von oben nach unten
/// `(255,240,0) (255,255,105) (255,255,105) (255,255,105) (255,240,0)` -
/// die 240 kommt nur zustande, wenn `sample[8]` bei (4,0) noch den Wert
/// von (4,1) aus dem vorigen Schleifendurchlauf traegt.
pub fn laplacian(s: &Surface) -> Surface {
    /// Der Fuellwert fuer fehlende Nachbarn - pygames `LAPLACIAN_NUM`.
    const LAPLACIAN_NUM: u32 = 0xFFFF_FFFF;

    let (w, h) = s.get_size();
    if w == 0 || h == 0 {
        return neu_wie(s, (w, h));
    }
    let q = lies_roh(s);
    let hat_alpha = s.has_alpha();
    let (wi, hi) = (w as i32, h as i32);
    let hole = |x: i32, y: i32| q[(y as usize) * (w as usize) + x as usize];
    // `PG_GetRGBA`: ohne Alphakanal kommt immer 255 zurueck, ganz gleich,
    // was im vierten Byte steht.
    let zerlege = |v: u32| -> [i32; 4] {
        let [b, g, r, a] = v.to_le_bytes();
        [
            r as i32,
            g as i32,
            b as i32,
            if hat_alpha { a as i32 } else { 255 },
        ]
    };

    // Bewusst **ausserhalb** der Schleifen: genau das macht die Eigenart
    // Nummer 3 aus. In C ist das ein lokales `Uint32 sample[9]`, das
    // zwischen den Pixeln nicht zurueckgesetzt wird.
    //
    // Die Startwerte sind der uninitialisierte Stapelspeicher aus der
    // C-Quelle. Er wird an genau zwei Stellen gelesen, beide am
    // allerersten Pixel (0,0) einer Flaeche mit mehr als einer Zeile:
    // `sample[6]` immer, `sample[8]` nur bei genau einem Pixel Breite.
    // Nachgemessen an echtem pygame mit Faellen, in denen das Ergebnis
    // nicht ans Ende des Bereichs stoesst:
    //
    // * 2x5, Pixel (0,0), Mitte 200, Nachbarn 153/154/155:
    //   `8*200 - 1020 - 3*n - erg` ergibt fuer alle drei Kanaele **0**
    //   -> `sample[6] == 0x00000000`.
    // * 1x5, Pixel (0,2), Mitte 200, Nachbarn 170/171/172: dieselbe
    //   Rechnung ergibt `(0, 0, 1)` -> `sample[8] == 0x00000001`.
    //
    // Ueber drei frische Prozesse und 50 Wiederholungen mit
    // zwischengeschobenen anderen Aufrufen blieb das gleich. Es ist
    // trotzdem undefiniertes Verhalten - auf einer anderen Maschine oder
    // mit einem anderen Uebersetzer kann dort etwas anderes stehen.
    // Betroffen ist ein einziger Pixel je Aufruf.
    let mut sample = [0u32, 0, 0, 0, 0, 0, 0, 0, 1];
    let mut ziel = Vec::with_capacity((w * h) as usize);

    for y in 0..hi {
        for x in 0..wi {
            if y > 0 {
                if x > 0 {
                    sample[0] = hole(x - 1, y - 1);
                }
                sample[1] = hole(x, y - 1);
                if x + 1 < wi {
                    sample[2] = hole(x + 1, y - 1);
                }
            } else {
                sample[0] = LAPLACIAN_NUM;
                sample[1] = LAPLACIAN_NUM;
                sample[2] = LAPLACIAN_NUM;
            }
            sample[3] = if x > 0 { hole(x - 1, y) } else { LAPLACIAN_NUM };
            // Nicht der Mittelpixel, sondern eine harte Null - siehe oben.
            sample[4] = 0;
            sample[5] = if x + 1 < wi {
                hole(x + 1, y)
            } else {
                LAPLACIAN_NUM
            };
            if y + 1 < hi {
                if x > 0 {
                    sample[6] = hole(x - 1, y + 1);
                }
                sample[7] = hole(x, y + 1);
                if x + 1 < wi {
                    sample[8] = hole(x + 1, y + 1);
                }
            } else {
                sample[6] = LAPLACIAN_NUM;
                sample[7] = LAPLACIAN_NUM;
                sample[8] = LAPLACIAN_NUM;
            }

            let mut total = [0i32; 4];
            for v in sample.iter() {
                let c = zerlege(*v);
                for (t, k) in total.iter_mut().zip(c.iter()) {
                    *t += *k;
                }
            }

            let mitte = zerlege(hole(x, y));
            let mut acolor = [0u8; 4];
            for (i, a) in acolor.iter_mut().enumerate() {
                *a = (mitte[i] * 8 - total[i]).clamp(0, 255) as u8;
            }
            ziel.push(u32::from_le_bytes([
                acolor[2], acolor[1], acolor[0], acolor[3],
            ]));
        }
    }
    schreibe_roh(s, (w, h), &ziel)
}

// ---------------------------------------------------------------------------
//  solid_overlay
// ---------------------------------------------------------------------------

/// Jeden sichtbaren Pixel durch eine Farbe ersetzen
/// (`pygame.transform.solid_overlay`). Das Mittel der Wahl fuer
/// Scherenschnitt-Schatten und Aufleuchten.
///
/// **Der Sonderfall, ueber den jede Messreihe stolpert:** hat die Quelle
/// **keinen** Alphakanal, steigt pygame sofort aus -
///
/// ```c
/// if (!SDL_ISPIXELFORMAT_ALPHA(fmt->format)) {
///     return newsurf;
/// }
/// ```
///
/// und `newsurf` ist die eben erst angelegte, **leere** Flaeche. Das
/// Ergebnis ist also durchgehend `0x00000000`, nicht die Quelle und nicht
/// die Farbe. Gemessen: `solid_overlay(flaeche_ohne_alpha, rot)` liefert
/// lauter Nullen. Wer das fuer einen Fehler in der eigenen Messung haelt,
/// sucht lange.
///
/// Mit Alphakanal: jeder Pixel mit `a != 0` bekommt die Farbe, jeder mit
/// `a == 0` bleibt auf der Null der neuen Flaeche stehen (also
/// durchsichtig **schwarz**, nicht der Originalpixel). Mit `keep_alpha`
/// wird das Alpha der Farbe verworfen und das des Quellpixels eingesetzt.
///
/// Gemessen an einer 2x2-Flaeche mit den Alphawerten 0, 1, 128, 255 und
/// der Farbe `(255,9,8,77)`: ohne `keep_alpha` `00000000 4dff0908
/// 4dff0908 4dff0908`, mit `keep_alpha` `00000000 01ff0908 80ff0908
/// ffff0908`.
pub fn solid_overlay(s: &Surface, farbe: impl IntoColor, keep_alpha: bool) -> Surface {
    let (w, h) = s.get_size();
    let mut ziel = neu_wie(s, (w, h));
    if !s.has_alpha() {
        // Siehe oben - pygame gibt die leere Flaeche zurueck.
        return ziel;
    }
    let mut c = s.map_rgb(farbe);
    if keep_alpha {
        // `color &= ~fmt->Amask`
        c &= 0x00FF_FFFF;
    }
    let q = lies_roh(s);
    ziel.mit_pixeln(|puffer, zeilenbreite, (ox, oy)| {
        for y in 0..h as usize {
            for x in 0..w as usize {
                let a = q[y * w as usize + x] >> 24;
                if a != 0 {
                    puffer[(oy + y) * zeilenbreite + ox + x] = if keep_alpha {
                        c | (a << 24)
                    } else {
                        c
                    };
                }
            }
        }
    });
    ziel
}

// ---------------------------------------------------------------------------
//  hsl
// ---------------------------------------------------------------------------

/// Farbton, Saettigung und Helligkeit verschieben (`pygame.transform.hsl`).
///
/// `hue` in Grad (wird mit `fmod` auf 360 gebracht), `saturation` und
/// `lightness` zwischen -1 und 1; alles andere ist ein Fehler, genau wie
/// in pygame. Null heisst jeweils "nichts tun" - und zwar wirklich: die
/// C-Quelle prueft `if (h)`, `if (s)`, `if (l)` und ueberspringt den
/// Schritt sonst ganz.
///
/// Gerechnet wird in `f32` nach der bekannten Vorlage von
/// `gist.github.com/ciembor/1494530`, samt ihrer Rundungen:
///
/// ```c
/// float q = l < 0.5f ? l * (1 + s) : l + s - l * s;
/// ```
///
/// **Diese eine Zeile ist der Grund fuer ein sonst unerklaerliches 254.**
/// Bei `l = 1.0` und `s = 0.9945` ergibt `l + s - l*s` in Fliesskomma
/// nicht genau 1, sondern eine Winzigkeit darunter - mal 255 und
/// abgeschnitten wird daraus 254 statt 255. Gemessen an echtem pygame:
/// `hsl(flaeche, 400, 0.17, 1.0)` macht aus `(17,200,90,0)` nicht weiss,
/// sondern `(255,254,254)`. Wer die Zeile "aufraeumt", baut genau hier
/// eine Abweichung ein.
///
/// Weitere Messwerte (4x3, Pixel `(x*60, y*80, (x+y)*40)`, `hue=60`):
/// `(0,0,0) (60,20,0) (120,40,0)`.
pub fn hsl(
    s: &Surface,
    hue: f32,
    saturation: f32,
    lightness: f32,
) -> Result<Surface, &'static str> {
    if !(-1.0..=1.0).contains(&saturation) {
        return Err("saturation value must be between -1 and 1");
    }
    if !(-1.0..=1.0).contains(&lightness) {
        return Err("lightness value must be between -1 and 1");
    }
    // `h = (float)(fmodf(h, 360.0) / 360.0)` - aus Grad wird ein Anteil.
    let h_anteil = (hue % 360.0) / 360.0;

    let (w, ho) = s.get_size();
    let hat_alpha = s.has_alpha();
    let daten: Vec<u32> = lies_roh(s)
        .iter()
        .map(|v| {
            let [b, g, r, a] = v.to_le_bytes();
            let (mut sh, mut ss, mut sl) = rgb_nach_hsl(r, g, b);
            if h_anteil != 0.0 {
                sh += h_anteil;
                if sh > 1.0 {
                    sh -= 1.0;
                } else if sh < 0.0 {
                    sh += 1.0;
                }
            }
            if saturation != 0.0 {
                ss *= 1.0 + saturation;
                ss = ss.clamp(0.0, 1.0);
            }
            if lightness != 0.0 {
                sl = if lightness < 0.0 {
                    sl * (1.0 + lightness)
                } else {
                    sl * (1.0 - lightness) + lightness
                };
                sl = sl.clamp(0.0, 1.0);
            }
            let (nr, ng, nb) = hsl_nach_rgb(sh, ss, sl);
            // `if (fmt->Amask) dstp8[Aidx] = srcp8[Aidx];` - ohne
            // Alphakanal bleibt das vierte Byte der neuen Flaeche stehen.
            u32::from_le_bytes([nb, ng, nr, if hat_alpha { a } else { 0 }])
        })
        .collect();
    Ok(schreibe_roh(s, (w, ho), &daten))
}

/// pygames `RGB_to_HSL` - Byte fuer Byte, in `f32`.
fn rgb_nach_hsl(r: u8, g: u8, b: u8) -> (f32, f32, f32) {
    let r1 = r as f32 / 255.0;
    let g1 = g as f32 / 255.0;
    let b1 = b as f32 / 255.0;
    let min = r1.min(g1).min(b1);
    let max = r1.max(g1).max(b1);
    let delta = max - min;
    let l = (max + min) / 2.0;
    if delta == 0.0 {
        // Grau - pygame setzt Farbton und Saettigung beide auf null.
        return (0.0, 0.0, l);
    }
    let s = if l > 0.5 {
        delta / (2.0 - max - min)
    } else {
        delta / (max + min)
    };
    let mut h = if max == r1 {
        (g1 - b1) / delta + if g1 < b1 { 6.0 } else { 0.0 }
    } else if max == g1 {
        (b1 - r1) / delta + 2.0
    } else {
        (r1 - g1) / delta + 4.0
    };
    h /= 6.0;
    (h, s, l)
}

/// pygames `hue_to_rgb`.
fn ton_nach_kanal(p: f32, q: f32, t: f32) -> f32 {
    let mut t = t;
    if t < 0.0 {
        t += 1.0;
    }
    if t > 1.0 {
        t -= 1.0;
    }
    if t < 1.0 / 6.0 {
        return p + (q - p) * 6.0 * t;
    }
    if t < 1.0 / 2.0 {
        return q;
    }
    if t < 2.0 / 3.0 {
        return p + (q - p) * (2.0 / 3.0 - t) * 6.0;
    }
    p
}

/// pygames `HSL_to_RGB` - samt der Zeile, die aus 255 eine 254 macht.
fn hsl_nach_rgb(h: f32, s: f32, l: f32) -> (u8, u8, u8) {
    if s == 0.0 {
        let v = (l * 255.0) as u8;
        return (v, v, v);
    }
    // Genau so, nicht umgestellt: `l + s - l * s` ist bei l=1 nicht 1.
    let q = if l < 0.5 { l * (1.0 + s) } else { l + s - l * s };
    let p = 2.0 * l - q;
    (
        (ton_nach_kanal(p, q, h + 1.0 / 3.0) * 255.0) as u8,
        (ton_nach_kanal(p, q, h) * 255.0) as u8,
        (ton_nach_kanal(p, q, h - 1.0 / 3.0) * 255.0) as u8,
    )
}

// ---------------------------------------------------------------------------
//  chop
// ---------------------------------------------------------------------------

/// Einen Streifen herausschneiden und den Rest zusammenschieben
/// (`pygame.transform.chop`).
///
/// Nicht zu verwechseln mit `subsurface`: `chop` entfernt **Zeilen und
/// Spalten**, die das Rechteck beruehrt, und klebt die Reste aneinander.
/// Die neue Groesse ist `(w - rect.w, h - rect.h)` nach dem Zurechtstutzen
/// des Rechtecks.
///
/// Die Reihenfolge der Zurechtstutzung ist pygames eigene und macht den
/// Unterschied: erst wird `width` gekuerzt, falls `x + width` ueber den
/// Rand ragt, **danach** erst ein negatives `x` verrechnet.
///
/// ```c
/// if ((x + width) > src->w) width = MIN(MAX(src->w - x, 0), src->w);
/// if (x < 0) { width -= (-x); x = 0; }
/// ```
///
/// Gemessen: `chop(4x3, Rect(1,0,1,1))` ergibt `(3,2)`,
/// `chop(4x3, Rect(0,0,4,3))` ergibt `(0,0)`, und - der schoenste Fall -
/// `chop(4x3, Rect(1,1,-2,-2))` ergibt `(6,5)`: bei negativer Breite wird
/// die Flaeche **groesser**, das Original landet links oben, der Rest
/// bleibt leer.
pub fn chop(s: &Surface, rect: Rect) -> Surface {
    let (w, h) = (s.get_width() as i32, s.get_height() as i32);
    let (mut x, mut y) = (rect.x, rect.y);
    let (mut breite, mut hoehe) = (rect.w, rect.h);

    if x + breite > w {
        breite = (w - x).max(0).min(w);
    }
    if y + hoehe > h {
        hoehe = (h - y).max(0).min(h);
    }
    if x < 0 {
        breite -= -x;
        x = 0;
    }
    if y < 0 {
        hoehe -= -y;
        y = 0;
    }

    let zw = (w - breite).max(0) as u32;
    let zh = (h - hoehe).max(0) as u32;
    let mut ziel = neu_wie(s, (zw, zh));
    if w == 0 || h == 0 || zw == 0 || zh == 0 {
        return ziel;
    }
    let q = lies_roh(s);
    ziel.mit_pixeln(|puffer, zeilenbreite, (ox, oy)| {
        let mut zy = 0usize;
        for ly in 0..h {
            if ly >= y && ly < y + hoehe {
                continue;
            }
            if zy >= zh as usize {
                break;
            }
            let mut zx = 0usize;
            for lx in 0..w {
                if lx >= x && lx < x + breite {
                    continue;
                }
                if zx >= zw as usize {
                    break;
                }
                puffer[(oy + zy) * zeilenbreite + ox + zx] =
                    q[(ly as usize) * (w as usize) + lx as usize];
                zx += 1;
            }
            zy += 1;
        }
    });
    ziel
}

// ---------------------------------------------------------------------------
//  pixelate
// ---------------------------------------------------------------------------

/// Verpixeln (`pygame.transform.pixelate`).
///
/// Kein eigener Filter, sondern zweimal [`scale`]: erst auf
/// `round(groesse / pixel_size)` herunter, dann wieder auf die
/// Ausgangsgroesse hinauf. Weil beide Wege Naechster-Nachbar sind,
/// entstehen genau die harten Kaesten, die man erwartet.
///
/// `round` ist das kaufmaennische Runden aus C (von der Null weg), und die
/// Zwischengroesse ist mindestens 1:
///
/// ```c
/// int width = (int)round((double)src_surf->w / pixel_size);
/// if (width < 1) width = 1;
/// ```
///
/// Gemessen (4x3): `pixel_size=3` ergibt `round(1.33)=1` und `round(1)=1`,
/// also die ganze Flaeche in der Farbe des Pixels (2,1).
pub fn pixelate(s: &Surface, pixel_size: i32) -> Result<Surface, &'static str> {
    if pixel_size < 1 {
        return Err("pixel_size must be greater than 0");
    }
    let (w, h) = s.get_size();
    let klein_w = ((w as f64 / pixel_size as f64).round() as i32).max(1) as u32;
    let klein_h = ((h as f64 / pixel_size as f64).round() as i32).max(1) as u32;
    Ok(scale(&scale(s, (klein_w, klein_h)), (w, h)))
}

// ---------------------------------------------------------------------------
//  average_surfaces
// ---------------------------------------------------------------------------

/// Mehrere Flaechen uebereinanderlegen und mitteln
/// (`pygame.transform.average_surfaces`).
///
/// Gemittelt werden **nur** Rot, Gruen und Blau; das Alpha wird weder
/// gelesen noch gemittelt. Geschrieben wird mit `PG_MapRGB`, und das setzt
/// bei einer Flaeche mit Alphakanal jeden Pixel auf **undurchsichtig**.
/// Gemessen: zwei Flaechen `(10,20,30,40)` und `(200,100,50,200)` ergeben
/// `ff693c28`, also `(105,60,40,255)` - das Alpha 120 taucht nirgends auf.
///
/// Gerundet wird in `f32`: `(Uint8)(summe * (1/n) + 0.5f)`.
///
/// **Abweichung:** pygame nimmt die Groesse der **ersten** Flaeche und
/// liest alle weiteren mit dieser Groesse aus - ist eine davon kleiner,
/// liest es ueber deren Puffer hinaus (nachgemessen: `[3x3, 2x2]` liefert
/// ein 3x3-Ergebnis mit Speichermuell). Hier gibt es dafuer einen Fehler.
/// Groessere Flaechen sind erlaubt; von ihnen wird - wie in pygame - die
/// linke obere Ecke benutzt.
pub fn average_surfaces(flaechen: &[&Surface]) -> Result<Surface, &'static str> {
    let Some(erste) = flaechen.first() else {
        return Err("Needs to be given at least one surface.");
    };
    let (w, h) = erste.get_size();
    if flaechen
        .iter()
        .any(|f| f.get_width() < w || f.get_height() < h)
    {
        return Err("all surfaces need to be at least as large as the first");
    }
    let anzahl = flaechen.len();
    let mut summe = vec![[0u32; 3]; (w as usize) * (h as usize)];
    for f in flaechen {
        for y in 0..h as i32 {
            for x in 0..w as i32 {
                let [b, g, r, _] = f.get_at_mapped((x, y)).to_le_bytes();
                let z = &mut summe[(y as usize) * (w as usize) + x as usize];
                z[0] += r as u32;
                z[1] += g as u32;
                z[2] += b as u32;
            }
        }
    }
    // `div_inv = (float)(1.0L / num_surfaces)` - einmal geteilt, dann
    // multipliziert. Das ist nicht dasselbe wie eine Division je Pixel.
    let div_inv = (1.0f64 / anzahl as f64) as f32;
    let voll = if erste.has_alpha() { 0xFFu8 } else { 0 };
    let daten: Vec<u32> = summe
        .iter()
        .map(|z| {
            let m = |v: u32| (v as f32 * div_inv + 0.5f32) as u8;
            // `PG_MapRGB` setzt das Alpha auf undurchsichtig.
            u32::from_le_bytes([m(z[2]), m(z[1]), m(z[0]), voll])
        })
        .collect();
    Ok(schreibe_roh(erste, (w, h), &daten))
}

// ---------------------------------------------------------------------------
//  threshold
// ---------------------------------------------------------------------------

/// Pixel nach Farbaehnlichkeit zaehlen und ersetzen
/// (`pygame.transform.threshold`).
///
/// Verglichen wird **kanalweise und nur in RGB** - kein Abstand im
/// Farbraum, sondern `|a - b| <= schwelle` fuer jeden der drei Kanaele
/// einzeln. Das Alpha spielt nirgends mit: pygame bildet Such- und
/// Schwellfarbe erst auf das Format der Quelle ab und liest sie dann mit
/// `PG_GetRGB` zurueck.
///
/// * `ziel` - wird beschrieben (bei `verhalten == 0` nicht noetig).
/// * `such_farbe` - wogegen verglichen wird. Zusammen mit `such_flaeche`
///   muss sie `None` sein; ohne beides wird nach Schwarz gesucht
///   (gemessen - pygame laesst `color_search_color` dort auf 0 stehen).
/// * `schwelle` - Vorgabe `(0,0,0,255)`, also "muss genau passen".
/// * `setz_farbe` - Vorgabe `(0,0,0,255)`.
/// * `verhalten` - `0` nur zaehlen, `1` `setz_farbe` eintragen, `2` den
///   Pixel aus der Quelle (bzw. aus `such_flaeche`) uebernehmen.
/// * `umgekehrt` - `false` aendert die Pixel **ausserhalb** der Schwelle,
///   `true` die innerhalb.
///
/// Rueckgabe ist die Zahl der Pixel innerhalb der Schwelle - unabhaengig
/// davon, welche geschrieben wurden.
///
/// Gemessen an einer 3x1-Flaeche `(10,20,30) (100,100,100) (250,250,250)`
/// mit Suchfarbe `(10,20,30)`, Schwelle `(5,5,5)`, Setzfarbe `(1,2,3)`:
/// `verhalten=1` ergibt 1 Treffer und `(7,7,7) (1,2,3) (1,2,3)`,
/// `verhalten=2` ergibt `(7,7,7) (100,100,100) (250,250,250)`, und
/// `umgekehrt=true` ergibt `(1,2,3) (7,7,7) (7,7,7)`.
#[allow(clippy::too_many_arguments)]
pub fn threshold(
    ziel: Option<&mut Surface>,
    quelle: &Surface,
    such_farbe: Option<Color>,
    schwelle: Option<Color>,
    setz_farbe: Option<Color>,
    verhalten: i32,
    such_flaeche: Option<&Surface>,
    umgekehrt: bool,
) -> Result<u32, &'static str> {
    let (w, h) = quelle.get_size();
    if let Some(z) = &ziel {
        if z.get_size() != (w, h) {
            return Err("surf and dest_surf not the same size");
        }
    } else if verhalten != 0 {
        return Err("argument 1 must be a Surface, or None with set_behavior=0");
    }
    if let Some(f) = such_flaeche {
        if f.get_size() != (w, h) {
            return Err("surf and search_surf not the same size");
        }
        if such_farbe.is_some() {
            return Err("if search_surf is used, search_color should be None");
        }
    }
    // Ohne `such_flaeche` und ohne `such_farbe` bleibt pygames
    // `color_search_color` auf 0 stehen und `PG_GetRGB` macht daraus
    // Schwarz - kein Fehler. Gemessen: `threshold(ziel, q, None, ...)`
    // sucht nach (0,0,0).

    // Beide Farben laufen durch das Format der **Quelle** und wieder
    // zurueck - auf einer Flaeche ohne Alphakanal faellt das Alpha dabei
    // weg, was aber nichts aendert, weil ohnehin nur RGB verglichen wird.
    let zu_rgb = |c: Color| {
        let [b, g, r, _] = quelle.map_rgb(c).to_le_bytes();
        [r, g, b]
    };
    let such = zu_rgb(such_farbe.unwrap_or(Color::rgba(0, 0, 0, 255)));
    let grenze = zu_rgb(schwelle.unwrap_or(Color::rgba(0, 0, 0, 255)));
    let setz = quelle.map_rgb(setz_farbe.unwrap_or(Color::rgba(0, 0, 0, 255)));

    let q = lies_roh(quelle);
    let vergleich = such_flaeche.map(lies_roh);
    let mut treffer = 0u32;
    let mut schreibliste: Vec<(usize, usize, u32)> = Vec::new();

    for y in 0..h as usize {
        for x in 0..w as usize {
            let i = y * w as usize + x;
            let [sb, sg, sr, _] = q[i].to_le_bytes();
            let (anders, setz_wert) = match &vergleich {
                Some(v) => {
                    let [vb, vg, vr, _] = v[i].to_le_bytes();
                    ([vr, vg, vb], if verhalten == 2 { v[i] } else { setz })
                }
                None => (such, if verhalten == 2 { q[i] } else { setz }),
            };
            let drin = (anders[0] as i32 - sr as i32).abs() <= grenze[0] as i32
                && (anders[1] as i32 - sg as i32).abs() <= grenze[1] as i32
                && (anders[2] as i32 - sb as i32).abs() <= grenze[2] as i32;
            if drin {
                treffer += 1;
            }
            if verhalten != 0 && ((drin && umgekehrt) || (!drin && !umgekehrt)) {
                schreibliste.push((x, y, setz_wert));
            }
        }
    }

    if let Some(z) = ziel {
        let maske = if z.has_alpha() {
            0xFFFF_FFFFu32
        } else {
            0x00FF_FFFF
        };
        z.mit_pixeln(|puffer, zeilenbreite, (ox, oy)| {
            for (x, y, v) in schreibliste {
                puffer[(oy + y) * zeilenbreite + ox + x] = v & maske;
            }
        });
    }
    Ok(treffer)
}

// ---------------------------------------------------------------------------
//  smoothscale-Backend
// ---------------------------------------------------------------------------

/// Die zwei Rechenwege von [`smoothscale`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Rechenweg {
    /// `filter_*_ONLYC` - pygame nennt ihn `"GENERIC"`.
    Generic,
    /// `filter_*_SSE2` - pygame nennt ihn `"SSE2"` (auf ARM `"NEON"`, mit
    /// derselben Rechnung ueber `sse2neon.h`).
    Sse2,
}

/// Der gewaehlte Rechenweg: 0 = `GENERIC`, 1 = `SSE2`.
///
/// Ein Wert je Prozess wie pygames Modulzustand. Vorgabe ist `SSE2` auf
/// x86 und x86-64 (dort ist SSE2 Pflicht, und pygames `smoothscale_init`
/// nimmt es, sobald `SDL_HasSSE2()` ja sagt) und auf aarch64 (dort
/// heisst er `NEON`); ueberall sonst `GENERIC`.
static RECHENWEG: AtomicU8 = AtomicU8::new(if VEKTORWEG.is_some() { 1 } else { 0 });

/// Der Name, unter dem pygame auf dieser Maschine den Vektorweg fuehrt.
const VEKTORWEG: Option<&str> = if cfg!(any(target_arch = "x86", target_arch = "x86_64")) {
    Some("SSE2")
} else if cfg!(target_arch = "aarch64") {
    Some("NEON")
} else {
    None
};

fn rechenweg() -> Rechenweg {
    if RECHENWEG.load(Ordering::Relaxed) == 1 {
        Rechenweg::Sse2
    } else {
        Rechenweg::Generic
    }
}

/// Welcher Weg `smoothscale` rechnet (`pygame.transform.get_smoothscale_backend`).
///
/// Vorgabe ist **`"SSE2"`**, wie bei pygame auf jedem x86-Rechner (auf
/// aarch64 `"NEON"`, sonst `"GENERIC"`). Seit Runde 18 ist das keine
/// Behauptung mehr, sondern die Rechnung: [`smoothscale`] traegt beide
/// Wege, und dieser Name sagt, welcher laeuft.
///
/// Hier stand vorher immer `"GENERIC"` - mit der Begruendung, eine Zahl,
/// die man nicht einhalten kann, sei schlimmer als eine ehrliche. Der
/// Satz war richtig und die Voraussetzung falsch: `SSE2` ist
/// Ganzzahlarithmetik und genauso einzuhalten wie `GENERIC`.
pub fn get_smoothscale_backend() -> &'static str {
    match rechenweg() {
        Rechenweg::Sse2 => VEKTORWEG.unwrap_or("GENERIC"),
        Rechenweg::Generic => "GENERIC",
    }
}

/// Den Rechenweg umstellen (`pygame.transform.set_smoothscale_backend`).
///
/// Gemessen an echtem pygame-ce 2.5.7 unter Windows x86-64:
///
/// ```text
/// "GENERIC"  -> ok          "SSE2"   -> ok
/// "NEON"     -> ValueError: NEON not supported on this machine
/// "MMX"      -> ValueError: Unknown backend type MMX
/// "SSE"      -> ValueError: Unknown backend type SSE
/// "sse2", "", "GENERIC " -> ValueError: Unknown backend type ...
/// ```
///
/// `MMX` und `SSE` sind **unbekannt**, nicht "nicht unterstuetzt" - pygame
/// ist ohne `SCALE_MMX_SUPPORT` gebaut, und sein Ersatzzweig vergleicht
/// den **bisherigen** Weg (`st->filter_type`) statt des gewuenschten. Hier
/// stand das Gegenteil, und die Probe hatte nur `XYZ`, `NEON` und
/// `GENERIC` gefragt.
pub fn set_smoothscale_backend(backend: &str) -> Result<(), String> {
    match backend {
        "GENERIC" => {
            RECHENWEG.store(0, Ordering::Relaxed);
            Ok(())
        }
        "SSE2" | "NEON" if VEKTORWEG == Some(backend) => {
            RECHENWEG.store(1, Ordering::Relaxed);
            Ok(())
        }
        "SSE2" | "NEON" => Err(format!("{backend} not supported on this machine")),
        _ => Err(format!("Unknown backend type {backend}")),
    }
}

// ---------------------------------------------------------------------------
//  rotozoom
// ---------------------------------------------------------------------------

/// Drehen und Skalieren in einem Schritt (`pygame.transform.rotozoom`).
///
/// Das ist **nicht** `rotate` plus `smoothscale`, sondern ein eigener
/// Rechenweg aus `SDL_gfx` 2.0.3, den pygame in `src_c/rotozoom.c`
/// mitschleppt. Er unterscheidet sich in jedem Punkt von [`rotate`]:
///
/// * **Die Groesse** kommt aus einer halbierten Rechnung mit `ceil` statt
///   aus abgeschnittenen Eckpunkten - und weil `x = width / 2` eine
///   **ganzzahlige** Division ist, ist das Ergebnis immer **gerade**. Eine
///   4x3-Flaeche um 90 Grad gedreht wird `(4,4)`, nicht `(3,4)`; gemessen.
/// * **Der Rand** bleibt leer (`0x00000000`), nicht die Farbe von Pixel
///   (0,0) wie bei [`rotate`]. Die Zielflaeche wird angelegt und nur dort
///   beschrieben, wo Quellpixel liegen.
/// * **Es wird gemischt**, nicht gestempelt: vier Nachbarn, bilinear in
///   16.16-Festkomma.
/// * Das **Flaechen-Alpha** des Ergebnisses ist 255, nicht das der Quelle
///   (`SDL_SetSurfaceAlphaMod(rz_dst, SDL_ALPHA_OPAQUE)`); die
///   Schluesselfarbe wird uebernommen. Gemessen: eine Quelle mit
///   `set_alpha(77)` liefert ein Ergebnis mit `get_alpha() == 255`.
///
/// Bei `winkel == 0` (genauer: `|winkel| <= 0.001`) laeuft ein eigener,
/// reiner Zoom - auch der mischt bilinear, aber mit anderer Schrittweite
/// als [`smoothscale`], und er rundet die Zielgroesse **ab**
/// (`(int)(breite * zoom)`, mindestens 1).
///
/// Sonderfaelle, gemessen: `skalierung == 0` ergibt eine 0x0-Flaeche, eine
/// negative Skalierung wird wie 0.001 behandelt (4x3 mit -1.0 und 30 Grad
/// ergibt `(2,2)`, mit 0 Grad `(1,1)`).
pub fn rotozoom(s: &Surface, winkel: f32, skalierung: f32) -> Surface {
    let (w, h) = s.get_size();
    if skalierung == 0.0 || w == 0 || h == 0 {
        // `newsurf_fromsurf(surf, 0, 0)` - mit allen Eigenschaften der
        // Quelle, aber leer.
        return neu_wie(s, (0, 0));
    }
    let winkel = winkel as f64;
    // `if (zoom < VALUE_LIMIT) zoom = VALUE_LIMIT;` - gilt auch fuer
    // negative Werte, und **vor** der Winkelabfrage.
    let zoom = (skalierung as f64).max(0.001);
    let zoominv = 65536.0 / (zoom * zoom);

    if winkel.abs() > 0.001 {
        let (dw, dh, canglezoom, sanglezoom) = rotozoom_groesse(w as i32, h as i32, winkel, zoom);
        let isin = (sanglezoom * zoominv) as i32;
        let icos = (canglezoom * zoominv) as i32;
        roto_drehen(s, dw, dh, dw / 2, dh / 2, isin, icos)
    } else {
        // `zoomSurfaceSize`: abschneiden, mindestens 1.
        let dw = ((w as f64 * zoom) as i32).max(1);
        let dh = ((h as f64 * zoom) as i32).max(1);
        roto_zoomen(s, dw, dh)
    }
}

/// pygames `rotozoomSurfaceSizeTrig`.
///
/// Die beiden ganzzahligen Halbierungen `width / 2` und `height / 2` sind
/// kein Schreibfehler: bei ungeraden Kantenlaengen geht dabei ein halber
/// Pixel verloren, und genau deshalb wird eine 5x5-Flaeche bei 90 Grad zu
/// `(4,4)` statt `(6,6)` - gemessen.
fn rotozoom_groesse(w: i32, h: i32, winkel: f64, zoom: f64) -> (i32, i32, f64, f64) {
    let bogen = winkel * (std::f64::consts::PI / 180.0);
    let sanglezoom = bogen.sin() * zoom;
    let canglezoom = bogen.cos() * zoom;
    let x = (w / 2) as f64;
    let y = (h / 2) as f64;
    let (cx, cy) = (canglezoom * x, canglezoom * y);
    let (sx, sy) = (sanglezoom * x, sanglezoom * y);
    let groesster = |a: f64, b: f64, c: f64, d: f64| a.max(b).max(c).max(d);
    let halbe_breite = (groesster(
        (cx + sy).abs(),
        (cx - sy).abs(),
        (-cx + sy).abs(),
        (-cx - sy).abs(),
    )
    .ceil() as i32)
        .max(1);
    let halbe_hoehe = (groesster(
        (sx + cy).abs(),
        (sx - cy).abs(),
        (-sx + cy).abs(),
        (-sx - cy).abs(),
    )
    .ceil() as i32)
        .max(1);
    (
        2 * halbe_breite,
        2 * halbe_hoehe,
        canglezoom,
        sanglezoom,
    )
}

/// pygames `transformSurfaceRGBA` mit `smooth = 1`.
///
/// Alles in 32-Bit-Festkomma, und alles mit Ueberlauf wie in C - deshalb
/// die `wrapping_*`-Rechnungen. Die Zeile
///
/// ```c
/// ay = (cy << 16) - (isin * cx);
/// ```
///
/// benutzt `cx`, wo man `cy` erwartet. Das steht seit SDL_gfx 2.0.3 so da
/// und ist hier unveraendert uebernommen - wer es "berichtigt",
/// verschiebt das ganze Bild.
fn roto_drehen(s: &Surface, dw: i32, dh: i32, cx: i32, cy: i32, isin: i32, icos: i32) -> Surface {
    let (sw, sh) = (s.get_width() as i32, s.get_height() as i32);
    let q = lies_roh(s);
    let mut ziel = roto_ziel(s, (dw.max(0) as u32, dh.max(0) as u32));
    if dw <= 0 || dh <= 0 {
        return ziel;
    }

    let xd = (sw - dw).wrapping_shl(15);
    let yd = (sh - dh).wrapping_shl(15);
    let ax = (cx.wrapping_shl(16)).wrapping_sub(icos.wrapping_mul(cx));
    // Ja, `cx` - siehe oben.
    let ay = (cy.wrapping_shl(16)).wrapping_sub(isin.wrapping_mul(cx));
    let (letzte_x, letzte_y) = (sw - 1, sh - 1);
    let hole = |x: i32, y: i32| q[(y.clamp(0, letzte_y) as usize) * (sw as usize)
        + x.clamp(0, letzte_x) as usize];

    let mut daten = vec![0u32; (dw as usize) * (dh as usize)];
    for y in 0..dh {
        let dyz = cy - y;
        let mut sdx = ax
            .wrapping_add(isin.wrapping_mul(dyz))
            .wrapping_add(xd);
        let mut sdy = ay
            .wrapping_sub(icos.wrapping_mul(dyz))
            .wrapping_add(yd);
        for x in 0..dw {
            let dx = sdx >> 16;
            let dy = sdy >> 16;
            if dx >= -1 && dy >= -1 && dx < sw && dy < sh {
                // Die neun Faelle aus der C-Quelle. Ausserhalb liegende
                // Nachbarn werden nicht gemittelt, sondern durch den
                // naechsten Randpixel ersetzt - deshalb sehen gedrehte
                // Bilder an den Kanten hart aus.
                let (c00, c01, c10, c11) =
                    if dx >= 0 && dy >= 0 && dx < letzte_x && dy < letzte_y {
                        (
                            hole(dx, dy),
                            hole(dx + 1, dy),
                            hole(dx, dy + 1),
                            hole(dx + 1, dy + 1),
                        )
                    } else if dx == letzte_x && dy == letzte_y {
                        let v = hole(dx, dy);
                        (v, v, v, v)
                    } else if dx == -1 && dy == -1 {
                        let v = hole(0, 0);
                        (v, v, v, v)
                    } else if dx == -1 && dy == letzte_y {
                        let v = hole(0, dy);
                        (v, v, v, v)
                    } else if dx == letzte_x && dy == -1 {
                        let v = hole(dx, 0);
                        (v, v, v, v)
                    } else if dx == -1 {
                        let v = hole(0, dy);
                        (v, v, v, hole(0, dy + 1))
                    } else if dy == -1 {
                        let v = hole(dx, 0);
                        (v, v, v, hole(dx + 1, 0))
                    } else if dx == letzte_x {
                        let o = hole(dx, dy);
                        let u = hole(dx, dy + 1);
                        (o, o, u, u)
                    } else if dy == letzte_y {
                        let l = hole(dx, dy);
                        let r = hole(dx + 1, dy);
                        (l, r, r, r)
                    } else {
                        // Der Auffangfall aus der Quelle ("Probably should
                        // not get here").
                        let v = hole(0, 0);
                        (v, v, v, v)
                    };
                let ex = sdx & 0xffff;
                let ey = sdy & 0xffff;
                daten[(y as usize) * (dw as usize) + x as usize] =
                    roto_mischen(c00, c01, c10, c11, ex, ey);
            }
            sdx = sdx.wrapping_add(icos);
            sdy = sdy.wrapping_add(isin);
        }
    }
    roto_schreiben(&mut ziel, dw as u32, dh as u32, &daten);
    ziel
}

/// pygames `zoomSurfaceRGBA` mit `smooth = 1` - der Weg ohne Drehung.
///
/// Die Schrittweite ist `65536 * (quelle - 1) / ziel`, also einer weniger
/// als man denkt ("assume source dimension is one pixel smaller to avoid
/// overflow on right and bottom edge"). Der letzte Quellpixel wird damit
/// nie ganz erreicht - dasselbe Muster wie bei [`smoothscale`], aber mit
/// anderer Rundung.
fn roto_zoomen(s: &Surface, dw: i32, dh: i32) -> Surface {
    let (sw, sh) = (s.get_width() as i32, s.get_height() as i32);
    let q = lies_roh(s);
    let mut ziel = roto_ziel(s, (dw.max(0) as u32, dh.max(0) as u32));
    if dw <= 0 || dh <= 0 {
        return ziel;
    }
    let sx = (65536.0 * (sw - 1) as f64 / dw as f64) as i32;
    let sy = (65536.0 * (sh - 1) as f64 / dh as f64) as i32;

    // Die Schrittabelle: der ganzzahlige Anteil wird nach jedem Eintrag
    // weggeworfen, der Rest laeuft weiter.
    let tabelle = |n: i32, schritt: i32| {
        let mut t = Vec::with_capacity(n as usize + 1);
        let mut c = 0i32;
        for _ in 0..=n {
            t.push(c);
            c &= 0xffff;
            c = c.wrapping_add(schritt);
        }
        t
    };
    let sax = tabelle(dw, sx);
    let say = tabelle(dh, sy);

    let hole = |x: i32, y: i32| {
        q[(y.min(sh - 1) as usize) * (sw as usize) + x.min(sw - 1) as usize]
    };
    let mut daten = vec![0u32; (dw as usize) * (dh as usize)];
    let mut zeile = 0i32;
    for y in 0..dh {
        let ey = say[y as usize] & 0xffff;
        let mut spalte = 0i32;
        for x in 0..dw {
            let ex = sax[x as usize] & 0xffff;
            daten[(y as usize) * (dw as usize) + x as usize] = roto_mischen(
                hole(spalte, zeile),
                hole(spalte + 1, zeile),
                hole(spalte, zeile + 1),
                hole(spalte + 1, zeile + 1),
                ex,
                ey,
            );
            spalte += sax[x as usize + 1] >> 16;
        }
        zeile += say[y as usize + 1] >> 16;
    }
    roto_schreiben(&mut ziel, dw as u32, dh as u32, &daten);
    ziel
}

/// Die bilineare Mischung, Byte fuer Byte wie in `rotozoom.c`.
///
/// Das `& 0xff` in der Mitte und das Abschneiden auf `u8` am Ende sind
/// nicht dasselbe wie ein Klemmen: bei ungluecklichen Werten laeuft der
/// Wert um, statt am Rand stehen zu bleiben. So steht es dort.
fn roto_mischen(c00: u32, c01: u32, c10: u32, c11: u32, ex: i32, ey: i32) -> u32 {
    let (a, b, c, d) = (
        c00.to_le_bytes(),
        c01.to_le_bytes(),
        c10.to_le_bytes(),
        c11.to_le_bytes(),
    );
    let mut aus = [0u8; 4];
    for k in 0..4 {
        let t1 = ((((b[k] as i32 - a[k] as i32) * ex) >> 16) + a[k] as i32) & 0xff;
        let t2 = ((((d[k] as i32 - c[k] as i32) * ex) >> 16) + c[k] as i32) & 0xff;
        aus[k] = ((((t2 - t1) * ey) >> 16) + t1) as u8;
    }
    u32::from_le_bytes(aus)
}

/// Die Zielflaeche fuer `rotozoom`: frisches Format, Schluesselfarbe von
/// der Quelle, Flaechen-Alpha **255**.
fn roto_ziel(s: &Surface, size: (u32, u32)) -> Surface {
    let mut z = Surface::with_flags(size, s.get_flags());
    z.set_colorkey(s.get_colorkey());
    if s.has_alpha() {
        // `SDL_SetSurfaceAlphaMod(rz_dst, SDL_ALPHA_OPAQUE)` - ohne
        // Alphakanal meldet pygame hier `None`, weil der Mischmodus
        // `SDL_BLENDMODE_NONE` ist.
        z.set_alpha(Some(255));
    }
    z
}

/// Rohe Werte in eine bereits fertig eingerichtete Flaeche schreiben.
fn roto_schreiben(ziel: &mut Surface, w: u32, h: u32, daten: &[u32]) {
    let maske = if ziel.has_alpha() {
        0xFFFF_FFFFu32
    } else {
        0x00FF_FFFF
    };
    ziel.mit_pixeln(|puffer, zeilenbreite, (ox, oy)| {
        for y in 0..h as usize {
            for x in 0..w as usize {
                puffer[(oy + y) * zeilenbreite + ox + x] = daten[y * w as usize + x] & maske;
            }
        }
    });
}

// ---------------------------------------------------------------------------
//  Tests - die Zahlen darin stammen aus echtem pygame-ce 2.5.7
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::surface::SRCALPHA;

    /// Eine Zeile mit vier bekannten Farben - dieselbe Vorlage wie bei den
    /// Messungen gegen echtes pygame.
    fn zeile4() -> Surface {
        let mut s = Surface::new((4, 1));
        for (x, c) in [(0, 0, 0), (255, 0, 0), (0, 255, 0), (0, 0, 255)]
            .iter()
            .enumerate()
        {
            s.set_at((x as i32, 0), *c);
        }
        s
    }

    fn zeile(s: &Surface, y: i32) -> Vec<(u8, u8, u8)> {
        (0..s.get_width() as i32)
            .map(|x| {
                let c = s.get_at((x, y));
                (c.r, c.g, c.b)
            })
            .collect()
    }

    #[test]
    fn scale_trifft_die_gemessenen_quellindizes() {
        // pygame, 4 -> 7: die Quellindizes [0,0,1,1,2,3,3].
        let a = scale(&zeile4(), (7, 1));
        assert_eq!(a.get_size(), (7, 1));
        assert_eq!(
            zeile(&a, 0),
            vec![
                (0, 0, 0),
                (0, 0, 0),
                (255, 0, 0),
                (255, 0, 0),
                (0, 255, 0),
                (0, 0, 255),
                (0, 0, 255),
            ]
        );
    }

    #[test]
    fn scale_verkleinert_mit_halbem_versatz() {
        // pygame, 5 -> 1 nimmt die Mitte (Index 2), nicht den Anfang.
        let mut s = Surface::new((5, 1));
        for x in 0..5 {
            s.set_at((x, 0), (x as u8 * 10, 0, 0));
        }
        assert_eq!(scale(&s, (1, 1)).get_at((0, 0)).r, 20);
        // 4 -> 16 verdoppelt jeden Pixel viermal.
        assert_eq!(
            zeile(&scale(&zeile4(), (16, 1)), 0)[0..5].to_vec(),
            vec![(0, 0, 0), (0, 0, 0), (0, 0, 0), (0, 0, 0), (255, 0, 0)]
        );
    }

    #[test]
    fn scale_behaelt_alphakanal_und_leere_groessen() {
        let mut s = Surface::with_flags((4, 4), SRCALPHA);
        s.fill((10, 20, 30, 128));
        let g = scale(&s, (8, 8));
        assert!(g.has_alpha(), "das SRCALPHA-Flag muss erhalten bleiben");
        assert_eq!(g.get_at((3, 3)).as_tuple(), (10, 20, 30, 128));
        // Eine Zielgroesse mit 0 ergibt eine leere Flaeche, keinen Absturz.
        assert_eq!(scale(&s, (0, 5)).get_size(), (0, 5));
    }

    #[test]
    fn smoothscale_trifft_die_gemessene_zeile() {
        // Gemessen an echtem pygame-ce 2.5.7, 4 -> 8.
        let a = smoothscale(&zeile4(), (8, 1));
        assert_eq!(
            zeile(&a, 0),
            vec![
                (0, 0, 0),
                (95, 0, 0),
                (191, 0, 0),
                (223, 31, 0),
                (127, 127, 0),
                (31, 223, 0),
                (0, 191, 63),
                (0, 95, 159),
            ]
        );
    }

    #[test]
    fn smoothscale_mittelt_beim_verkleinern() {
        // Gemessen: vier Ecken (schwarz, rot, gruen, blau) auf ein Pixel
        // ergibt (63,63,63) - nicht (64,64,64). Der Kastenfilter schneidet ab.
        let mut s = Surface::with_flags((2, 2), SRCALPHA);
        s.set_at((0, 0), (0, 0, 0, 255));
        s.set_at((1, 0), (255, 0, 0, 255));
        s.set_at((0, 1), (0, 255, 0, 255));
        s.set_at((1, 1), (0, 0, 255, 255));
        assert_eq!(
            smoothscale(&s, (1, 1)).get_at((0, 0)).as_tuple(),
            (63, 63, 63, 255)
        );
    }

    #[test]
    fn smoothscale_auf_gleiche_groesse_ist_verlustfrei() {
        // pygame behandelt diesen Fall als reine Kopie.
        let mut s = Surface::new((5, 3));
        for y in 0..3 {
            for x in 0..5 {
                s.set_at((x, y), (x as u8 * 50, y as u8 * 80, 7));
            }
        }
        let a = smoothscale(&s, (5, 3));
        for y in 0..3 {
            assert_eq!(zeile(&a, y), zeile(&s, y), "Zeile {y} muss gleich sein");
        }
    }

    #[test]
    fn weichzeichnen_verwischt_eine_kante() {
        // Der Kniff der Spielesammlung: verkleinern und wieder aufblasen.
        let mut s = Surface::new((16, 16));
        s.fill((0, 0, 0));
        s.fill_rect((255, 255, 255), Rect::new(0, 0, 8, 16));
        let weich = smoothscale(&smoothscale(&s, (4, 4)), (16, 16));
        assert_eq!(weich.get_size(), (16, 16));
        let mitte = weich.get_at((8, 8)).r;
        assert!(
            mitte > 0 && mitte < 255,
            "an der Kante muss ein Zwischenwert stehen, war {mitte}"
        );
        assert_eq!(weich.get_at((0, 8)).r, 255, "links bleibt weiss");
        assert_eq!(weich.get_at((15, 8)).r, 0, "rechts bleibt schwarz");
    }

    #[test]
    fn flip_spiegelt_beide_achsen() {
        let mut s = Surface::new((3, 2));
        for y in 0..2 {
            for x in 0..3 {
                s.set_at((x, y), ((x * 10 + y) as u8, 0, 0));
            }
        }
        assert_eq!(flip(&s, true, false).get_at((0, 0)).r, 20);
        assert_eq!(flip(&s, false, true).get_at((0, 0)).r, 1);
        assert_eq!(flip(&s, true, true).get_at((0, 0)).r, 21);
        // Zweimal gespiegelt ist wieder das Original.
        let zurueck = flip(&flip(&s, true, true), true, true);
        assert_eq!(zeile(&zurueck, 0), zeile(&s, 0));
    }

    #[test]
    fn rotate_90_dreht_gegen_den_uhrzeigersinn() {
        // Gemessen an echtem pygame mit einer 3x2-Flaeche.
        let mut s = Surface::new((3, 2));
        for y in 0..2 {
            for x in 0..3 {
                s.set_at((x, y), ((x * 10 + y) as u8, 0, 0));
            }
        }
        let a = rotate(&s, 90.0);
        assert_eq!(a.get_size(), (2, 3));
        // dst(x,y) = src(w-1-y, x)
        assert_eq!(a.get_at((0, 0)).r, 20);
        assert_eq!(a.get_at((1, 0)).r, 21);
        assert_eq!(a.get_at((0, 2)).r, 0);
        // -90 Grad ist dasselbe wie 270 Grad.
        assert_eq!(
            zeile(&rotate(&s, -90.0), 0),
            zeile(&rotate(&s, 270.0), 0)
        );
        // Viermal 90 Grad ist wieder das Original - verlustfrei.
        let mut r = s.copy();
        for _ in 0..4 {
            r = rotate(&r, 90.0);
        }
        assert_eq!(r.get_size(), (3, 2));
        assert_eq!(zeile(&r, 1), zeile(&s, 1));
    }

    #[test]
    fn rotate_180_kippt_die_ecken() {
        let mut s = Surface::new((3, 2));
        s.fill((0, 0, 0));
        s.set_at((0, 0), (255, 0, 0));
        let a = rotate(&s, 180.0);
        assert_eq!(a.get_size(), (3, 2));
        assert_eq!(a.get_at((2, 1)).r, 255);
        assert_eq!(a.get_at((0, 0)).r, 0);
    }

    #[test]
    fn rotate_schraeg_wird_groesser_und_fuellt_die_ecken() {
        // Gemessen: 12x8 bei 45 Grad ergibt 14x14 (abgeschnitten, nicht
        // aufgerundet), und die Ecken bekommen die Farbe von Pixel (0,0).
        let mut s = Surface::new((12, 8));
        s.fill((40, 40, 60));
        s.set_at((0, 0), (255, 255, 200));
        let a = rotate(&s, 45.0);
        assert_eq!(a.get_size(), (14, 14));
        assert_eq!(a.get_at((0, 0)).as_tuple(), (255, 255, 200, 255));
        assert_eq!(a.get_at((7, 7)).as_tuple(), (40, 40, 60, 255));
        // Mit Alphakanal sind die Ecken stattdessen durchsichtig.
        let mut t = Surface::with_flags((12, 8), SRCALPHA);
        t.fill((40, 40, 60, 255));
        assert_eq!(rotate(&t, 45.0).get_at((0, 0)).a, 0);
    }

    #[test]
    fn grayscale_trifft_die_opencv_festkommaformel() {
        // Gemessen: (200,100,50) wird 125, das Alpha bleibt stehen.
        let mut s = Surface::with_flags((3, 1), SRCALPHA);
        s.set_at((0, 0), (200, 100, 50, 128));
        s.set_at((1, 0), (255, 255, 255, 255));
        s.set_at((2, 0), (0, 0, 0, 0));
        let g = grayscale(&s);
        assert_eq!(g.get_at((0, 0)).as_tuple(), (125, 125, 125, 128));
        assert_eq!(g.get_at((1, 0)).as_tuple(), (255, 255, 255, 255));
        assert_eq!(g.get_at((2, 0)).as_tuple(), (0, 0, 0, 0));
    }

    #[test]
    fn average_color_mittelt_wie_pygame() {
        // Gemessen: (10,20,30,0) und (200,100,50,255) ergibt ohne
        // Alphagewichtung (105,60,40,127), mit (200,100,50,127).
        let mut s = Surface::with_flags((2, 1), SRCALPHA);
        s.set_at((0, 0), (10, 20, 30, 0));
        s.set_at((1, 0), (200, 100, 50, 255));
        assert_eq!(
            average_color(&s, None, false).as_tuple(),
            (105, 60, 40, 127)
        );
        assert_eq!(
            average_color(&s, None, true).as_tuple(),
            (200, 100, 50, 127)
        );
        // Ohne Alphakanal gibt es nichts zu mitteln - Alpha 0, wie pygame.
        let mut o = Surface::new((2, 1));
        o.set_at((0, 0), (10, 20, 30));
        o.set_at((1, 0), (200, 100, 50));
        assert_eq!(average_color(&o, None, false).as_tuple(), (105, 60, 40, 0));
    }

    #[test]
    fn scale2x_verdoppelt_und_glaettet_schraegen() {
        let mut s = Surface::new((3, 2));
        s.fill((0, 0, 0));
        s.set_at((0, 0), (255, 0, 0));
        s.set_at((1, 1), (255, 0, 0));
        let a = scale2x(&s);
        assert_eq!(a.get_size(), (6, 4));
        // Gemessen an echtem pygame. Gut zu sehen: die Treppe wird nicht
        // einfach verdoppelt, sondern an (2,1) und (1,2) ausgefuellt - und
        // dafuer faellt (1,1) weg. Genau das ist Scale2x.
        let rot: Vec<Vec<u8>> = (0..4)
            .map(|y| (0..6).map(|x| a.get_at((x, y)).r).collect())
            .collect();
        assert_eq!(
            rot,
            vec![
                vec![255, 255, 0, 0, 0, 0],
                vec![255, 0, 255, 0, 0, 0],
                vec![0, 255, 255, 255, 0, 0],
                vec![0, 0, 255, 255, 0, 0],
            ]
        );
    }

    #[test]
    fn skalieren_um_faktoren_schneidet_ab() {
        let s = Surface::new((5, 3));
        // 5 mal 2,4 ist 12,0 -> 12; 3 mal 2,4 ist 7,2 -> 7.
        assert_eq!(scale_by(&s, 2.4, 2.4).get_size(), (12, 7));
        assert_eq!(smoothscale_by(&s, 1.5, 0.5).get_size(), (7, 1));
        assert_eq!(weichzeichnen(&s, 1.0).get_size(), (5, 3));
    }

    // -----------------------------------------------------------------
    //  Die Vorlage fuer alles Folgende: 4x3 mit `(x*60, y*80, (x+y)*40)`.
    //  Dieselbe Flaeche steckt in jeder Messung gegen echtes pygame.
    // -----------------------------------------------------------------

    fn basis() -> Surface {
        let mut s = Surface::new((4, 3));
        for y in 0..3i32 {
            for x in 0..4i32 {
                s.set_at((x, y), ((x * 60) as u8, (y * 80) as u8, ((x + y) * 40) as u8));
            }
        }
        s
    }

    /// Alle Pixel als RGB-Zeilen - so stehen auch die Messwerte da.
    fn bild(s: &Surface) -> Vec<Vec<(u8, u8, u8)>> {
        (0..s.get_height() as i32)
            .map(|y| {
                (0..s.get_width() as i32)
                    .map(|x| {
                        let c = s.get_at((x, y));
                        (c.r, c.g, c.b)
                    })
                    .collect()
            })
            .collect()
    }

    #[test]
    fn invert_dreht_nur_die_farbe_um() {
        // Gemessen an echtem pygame: die erste Zeile der Vorlage.
        assert_eq!(
            bild(&invert(&basis()))[0],
            vec![
                (255, 255, 255),
                (195, 255, 215),
                (135, 255, 175),
                (75, 255, 135)
            ]
        );
        // Das Alpha bleibt stehen, auch ein halbes.
        let mut s = Surface::with_flags((2, 1), SRCALPHA);
        s.set_at((0, 0), (1, 2, 3, 255));
        s.set_at((1, 0), (128, 128, 128, 7));
        let a = invert(&s);
        assert_eq!(a.get_at((0, 0)).as_tuple(), (254, 253, 252, 255));
        assert_eq!(a.get_at((1, 0)).as_tuple(), (127, 127, 127, 7));
    }

    #[test]
    fn box_blur_mittelt_mit_gleitender_summe() {
        // Gemessen, radius 1, mit Randwiederholung.
        assert_eq!(
            bild(&box_blur(&basis(), 1, true).unwrap())[0],
            vec![(20, 26, 26), (60, 26, 53), (120, 26, 93), (160, 26, 119)]
        );
        // Ohne Randwiederholung wird der Rand dunkler - pygame teilt auch
        // dort durch 2*radius+1, obwohl weniger Pixel beitragen.
        assert_eq!(
            bild(&box_blur(&basis(), 1, false).unwrap())[0],
            vec![(13, 17, 17), (40, 26, 39), (80, 26, 66), (66, 17, 53)]
        );
        // radius 0 ist die Kopie, negativ ist ein Fehler.
        assert_eq!(bild(&box_blur(&basis(), 0, true).unwrap()), bild(&basis()));
        assert!(box_blur(&basis(), -1, true).is_err());
    }

    #[test]
    fn gaussian_blur_trifft_die_glockenkurve() {
        // Gemessen, radius 1 (also Sigma 1, Kern bis 2).
        assert_eq!(
            bild(&gaussian_blur(&basis(), 1, true).unwrap())[0],
            vec![(21, 28, 28), (63, 28, 56), (116, 28, 91), (158, 28, 120)]
        );
        // radius 0 ist **nicht** die Kopie, sondern schwarz: in der
        // C-Quelle wird die Glockenkurve dort zu NaN, und `(Uint8)NaN`
        // ist 0. Gemessen an echtem pygame.
        let null = gaussian_blur(&basis(), 0, true).unwrap();
        assert!(bild(&null).iter().all(|z| z.iter().all(|p| *p == (0, 0, 0))));
        // Und dasselbe, wenn der Radius erst durch die Klemme zu 0 wird:
        // auf einer nur ein Pixel hohen Flaeche gilt das fuer jeden
        // Radius groesser 1 (gemessen an 1x5 mit radius 3).
        let mut schmal = Surface::new((1, 5));
        for y in 0..5 {
            schmal.set_at((0, y), (200, 100, 50));
        }
        let g = gaussian_blur(&schmal, 3, true).unwrap();
        assert!((0..5).all(|y| g.get_at((0, y)).r == 0));
    }

    #[test]
    fn laplacian_faerbt_den_rand_weiss_und_zaehlt_die_mitte_doppelt() {
        // Ein gleichmaessiger Verlauf loescht sich im Inneren aus - bis
        // auf die rechte Spalte, wo `sample[5]` weiss ist. Gemessen:
        // (3,1) wird (105,0,25), alles andere schwarz.
        let l = bild(&laplacian(&basis()));
        assert_eq!(l[1][3], (105, 0, 25));
        assert_eq!(l[0], vec![(0, 0, 0); 4]);
        assert_eq!(l[1][0..3].to_vec(), vec![(0, 0, 0); 3]);

        // Der Fall, der die stehengebliebenen `sample`-Werte zeigt:
        // 5x5 schwarz, Spalte 0 und 4 hell. Gemessen an echtem pygame.
        let mut s = Surface::new((5, 5));
        s.fill((0, 0, 0));
        for i in 0..5 {
            s.set_at((0, i), (255, 200, 100));
            s.set_at((4, i), (250, 210, 90));
        }
        let a = bild(&laplacian(&s));
        assert_eq!(
            a.iter().map(|z| z[0]).collect::<Vec<_>>(),
            vec![
                (255, 255, 0),
                (255, 255, 90),
                (255, 255, 255),
                (255, 255, 255),
                (255, 255, 0)
            ]
        );
        // Die 240 oben und unten rechts entsteht nur, weil `sample[8]`
        // dort noch den Wert aus dem vorigen Durchlauf traegt.
        assert_eq!(
            a.iter().map(|z| z[4]).collect::<Vec<_>>(),
            vec![
                (255, 240, 0),
                (255, 255, 105),
                (255, 255, 105),
                (255, 255, 105),
                (255, 240, 0)
            ]
        );
    }

    #[test]
    fn solid_overlay_gibt_ohne_alphakanal_eine_leere_flaeche() {
        // Der Sonderfall: pygame steigt aus, bevor es etwas eintraegt -
        // heraus kommt die frisch angelegte, schwarze Flaeche. Gemessen.
        let a = solid_overlay(&basis(), (255, 0, 0), false);
        assert_eq!(a.get_size(), (4, 3));
        assert!(bild(&a).iter().all(|z| z.iter().all(|p| *p == (0, 0, 0))));

        // Mit Alphakanal: gemessen an den Alphawerten 0, 1, 128, 255.
        let mut s = Surface::with_flags((2, 2), SRCALPHA);
        s.set_at((0, 0), (10, 20, 30, 0));
        s.set_at((1, 0), (10, 20, 30, 1));
        s.set_at((0, 1), (10, 20, 30, 128));
        s.set_at((1, 1), (10, 20, 30, 255));
        let o = solid_overlay(&s, (255, 9, 8, 77), false);
        assert_eq!(o.get_at((0, 0)).as_tuple(), (0, 0, 0, 0));
        assert_eq!(o.get_at((1, 0)).as_tuple(), (255, 9, 8, 77));
        assert_eq!(o.get_at((1, 1)).as_tuple(), (255, 9, 8, 77));
        let k = solid_overlay(&s, (255, 9, 8, 77), true);
        assert_eq!(k.get_at((0, 0)).as_tuple(), (0, 0, 0, 0));
        assert_eq!(k.get_at((1, 0)).as_tuple(), (255, 9, 8, 1));
        assert_eq!(k.get_at((0, 1)).as_tuple(), (255, 9, 8, 128));
        assert_eq!(k.get_at((1, 1)).as_tuple(), (255, 9, 8, 255));
    }

    #[test]
    fn hsl_verschiebt_den_farbton_und_behaelt_die_rundung() {
        // Gemessen, hue = 60 Grad.
        let a = bild(&hsl(&basis(), 60.0, 0.0, 0.0).unwrap());
        assert_eq!(a[0], vec![(0, 0, 0), (60, 20, 0), (120, 40, 0), (180, 60, 0)]);
        assert_eq!(a[1][1], (59, 59, 80));

        // Die Zeile `q = l + s - l * s` bei l = 1: statt weiss kommt
        // (255,254,254) heraus. Gemessen an echtem pygame.
        let mut s = Surface::with_flags((1, 1), SRCALPHA);
        s.set_at((0, 0), (17, 200, 90, 0));
        assert_eq!(
            hsl(&s, 400.0, 0.17, 1.0).unwrap().get_at((0, 0)).as_tuple(),
            (255, 254, 254, 0)
        );

        // Die Grenzen sind einschliesslich - nur darueber hinaus ist Schluss.
        assert!(hsl(&basis(), 0.0, 1.0, -1.0).is_ok());
        assert!(hsl(&basis(), 0.0, 1.5, 0.0).is_err());
        assert!(hsl(&basis(), 0.0, 0.0, -1.5).is_err());
    }

    #[test]
    fn chop_schneidet_zeilen_und_spalten_heraus() {
        // Gemessen: Spalte 1 und Zeile 0 fallen weg.
        let a = chop(&basis(), Rect::new(1, 0, 1, 1));
        assert_eq!(a.get_size(), (3, 2));
        assert_eq!(bild(&a)[0], vec![(0, 80, 40), (120, 80, 120), (180, 80, 160)]);
        // Alles weg heisst nichts uebrig.
        assert_eq!(chop(&basis(), Rect::new(0, 0, 4, 3)).get_size(), (0, 0));
        // Und der Fall, den niemand erwartet: bei negativer Breite wird
        // die Flaeche **groesser**. Gemessen: (6,5).
        let g = chop(&basis(), Rect::new(1, 1, -2, -2));
        assert_eq!(g.get_size(), (6, 5));
        assert_eq!(bild(&g)[0][0..4].to_vec(), bild(&basis())[0]);
        assert_eq!(bild(&g)[4], vec![(0, 0, 0); 6]);
    }

    #[test]
    fn pixelate_macht_kaesten() {
        // Gemessen: bei 2 bleibt je ein Pixel aus jedem 2x2-Block uebrig,
        // bei 3 die ganze Flaeche in der Farbe von (2,1).
        assert_eq!(
            bild(&pixelate(&basis(), 2).unwrap())[0],
            vec![(60, 0, 40), (60, 0, 40), (180, 0, 120), (180, 0, 120)]
        );
        assert!(bild(&pixelate(&basis(), 3).unwrap())
            .iter()
            .all(|z| z.iter().all(|p| *p == (120, 80, 120))));
        assert!(pixelate(&basis(), 0).is_err());
    }

    #[test]
    fn average_surfaces_mittelt_nur_die_farbe() {
        // Gemessen: das Alpha wird weder gemittelt noch uebernommen -
        // das Ergebnis ist undurchsichtig.
        let mut a = Surface::with_flags((2, 1), SRCALPHA);
        a.fill((10, 20, 30, 40));
        let mut b = Surface::with_flags((2, 1), SRCALPHA);
        b.fill((200, 100, 50, 200));
        let m = average_surfaces(&[&a, &b]).unwrap();
        assert_eq!(m.get_at((0, 0)).as_tuple(), (105, 60, 40, 255));
        // Drei Flaechen, gemessen: (70,40,27).
        let mut c = Surface::with_flags((2, 1), SRCALPHA);
        c.fill((1, 1, 1, 1));
        assert_eq!(
            average_surfaces(&[&a, &b, &c]).unwrap().get_at((1, 0)).as_tuple(),
            (70, 40, 27, 255)
        );
        assert!(average_surfaces(&[]).is_err());
    }

    #[test]
    fn threshold_zaehlt_und_setzt() {
        let mut q = Surface::new((3, 1));
        q.set_at((0, 0), (10, 20, 30));
        q.set_at((1, 0), (100, 100, 100));
        q.set_at((2, 0), (250, 250, 250));
        let such = Some(Color::rgba(10, 20, 30, 255));
        let schwelle = Some(Color::rgba(5, 5, 5, 255));
        let setz = Some(Color::rgba(1, 2, 3, 255));

        // verhalten 1: was **nicht** passt, bekommt die Setzfarbe.
        let mut d = Surface::new((3, 1));
        d.fill((7, 7, 7));
        let n = threshold(Some(&mut d), &q, such, schwelle, setz, 1, None, false).unwrap();
        assert_eq!(n, 1);
        assert_eq!(bild(&d)[0], vec![(7, 7, 7), (1, 2, 3), (1, 2, 3)]);

        // verhalten 2 uebernimmt stattdessen den Quellpixel.
        let mut d2 = Surface::new((3, 1));
        d2.fill((7, 7, 7));
        threshold(Some(&mut d2), &q, such, schwelle, setz, 2, None, false).unwrap();
        assert_eq!(
            bild(&d2)[0],
            vec![(7, 7, 7), (100, 100, 100), (250, 250, 250)]
        );

        // umgekehrt: jetzt trifft es das, was passt.
        let mut d3 = Surface::new((3, 1));
        d3.fill((7, 7, 7));
        threshold(Some(&mut d3), &q, such, schwelle, setz, 1, None, true).unwrap();
        assert_eq!(bild(&d3)[0], vec![(1, 2, 3), (7, 7, 7), (7, 7, 7)]);

        // Nur zaehlen, ohne Zielflaeche.
        assert_eq!(
            threshold(None, &q, such, schwelle, None, 0, None, false).unwrap(),
            1
        );
        // Eine Zielflaeche anderer Groesse ist ein Fehler.
        let mut falsch = Surface::new((5, 5));
        assert!(threshold(Some(&mut falsch), &q, such, schwelle, setz, 1, None, false).is_err());

        // Ohne Suchfarbe und ohne Suchflaeche wird nach Schwarz gesucht -
        // gemessen, kein Fehler.
        let mut s = Surface::new((3, 1));
        s.set_at((0, 0), (0, 0, 0));
        s.set_at((1, 0), (1, 1, 1));
        s.set_at((2, 0), (250, 250, 250));
        let mut d4 = Surface::new((3, 1));
        d4.fill((7, 7, 7));
        let n4 = threshold(
            Some(&mut d4),
            &s,
            None,
            Some(Color::rgba(0, 0, 0, 255)),
            setz,
            1,
            None,
            false,
        )
        .unwrap();
        assert_eq!(n4, 1);
        assert_eq!(bild(&d4)[0], vec![(7, 7, 7), (1, 2, 3), (1, 2, 3)]);

        // Suchflaeche und Suchfarbe zusammen sind ein Fehler - wie in pygame.
        let such_f = q.copy();
        let mut d5 = Surface::new((3, 1));
        assert!(threshold(
            Some(&mut d5),
            &q,
            such,
            schwelle,
            setz,
            1,
            Some(&such_f),
            false
        )
        .is_err());

        // Mit Suchflaeche: jeder Pixel wird gegen seinen Partner gehalten.
        let mut partner = Surface::new((3, 1));
        partner.set_at((0, 0), (10, 20, 30));
        partner.set_at((1, 0), (200, 200, 200));
        partner.set_at((2, 0), (250, 250, 250));
        let mut d6 = Surface::new((3, 1));
        d6.fill((7, 7, 7));
        let n6 = threshold(
            Some(&mut d6),
            &q,
            None,
            schwelle,
            setz,
            1,
            Some(&partner),
            false,
        )
        .unwrap();
        assert_eq!(n6, 2);
        assert_eq!(bild(&d6)[0], vec![(7, 7, 7), (1, 2, 3), (7, 7, 7)]);
    }

    #[test]
    fn rotozoom_dreht_mit_gerader_zielgroesse() {
        // Gemessen: eine 4x3-Flaeche um 90 Grad wird (4,4) - nicht (3,4).
        // Schuld ist die ganzzahlige Halbierung `x = width / 2`.
        let a = rotozoom(&basis(), 90.0, 1.0);
        assert_eq!(a.get_size(), (4, 4));
        // Die erste Zeile bleibt leer (kein Quellpixel faellt darauf),
        // die zweite beginnt mit dem Pixel (3,0) der Vorlage.
        assert_eq!(bild(&a)[0], vec![(0, 0, 0); 4]);
        assert_eq!(
            bild(&a)[1],
            vec![(180, 0, 120), (180, 40, 140), (180, 120, 180), (180, 160, 200)]
        );
        assert_eq!(bild(&a)[3][0], (60, 0, 40));

        // Ohne Drehung laeuft der reine Zoom - gemessen, Faktor 2.
        let z = rotozoom(&basis(), 0.0, 2.0);
        assert_eq!(z.get_size(), (8, 6));
        assert_eq!(
            bild(&z)[0][0..4].to_vec(),
            vec![(0, 0, 0), (22, 0, 15), (45, 0, 30), (67, 0, 45)]
        );
        // Verkleinern schneidet die Zielgroesse ab, mindestens 1.
        let k = rotozoom(&basis(), 0.0, 0.5);
        assert_eq!(k.get_size(), (2, 1));
        assert_eq!(bild(&k)[0], vec![(0, 0, 0), (90, 0, 60)]);

        // Schraeg: 45 Grad ergibt (6,6), die erste Zeile bleibt leer.
        let s45 = rotozoom(&basis(), 45.0, 1.0);
        assert_eq!(s45.get_size(), (6, 6));
        assert_eq!(bild(&s45)[0], vec![(0, 0, 0); 6]);
        assert_eq!(
            bild(&s45)[3],
            vec![
                (0, 0, 0),
                (35, 6, 26),
                (77, 63, 82),
                (120, 120, 140),
                (165, 160, 190),
                (180, 160, 200)
            ]
        );

        // Sonderfaelle, gemessen: Faktor 0 ergibt nichts, ein negativer
        // Faktor wird wie 0,001 behandelt.
        assert_eq!(rotozoom(&basis(), 30.0, 0.0).get_size(), (0, 0));
        assert_eq!(rotozoom(&basis(), 30.0, -1.0).get_size(), (2, 2));
        assert_eq!(rotozoom(&basis(), 0.0, -1.0).get_size(), (1, 1));
    }

    #[test]
    fn rotozoom_setzt_das_flaechen_alpha_auf_voll() {
        // pygame ruft `SDL_SetSurfaceAlphaMod(rz_dst, SDL_ALPHA_OPAQUE)` -
        // das Flaechen-Alpha der Quelle ueberlebt also **nicht**.
        // Gemessen: eine Quelle mit set_alpha(77) liefert 255.
        let mut s = Surface::with_flags((4, 3), SRCALPHA);
        s.fill((10, 20, 30, 255));
        s.set_alpha(Some(77));
        s.set_colorkey(Some((10, 20, 30)));
        let a = rotozoom(&s, 45.0, 1.0);
        assert_eq!(a.get_alpha(), Some(255));
        assert_eq!(a.get_colorkey().map(|c| c.as_tuple()), Some((10, 20, 30, 255)));
        // Ohne Alphakanal meldet pygame dort `None` - der Mischmodus der
        // frischen Flaeche ist `SDL_BLENDMODE_NONE`.
        let mut o = Surface::new((4, 3));
        o.set_alpha(Some(77));
        assert_eq!(rotozoom(&o, 45.0, 1.0).get_alpha(), None);
    }

    #[test]
    fn neu_wie_folgt_pg_surface_srcalpha() {
        // Eine Flaeche **ohne** Alphakanal mit `set_alpha(255)` meldet in
        // pygame 255, das Ergebnis jeder transform-Funktion aber `None`:
        // `_PgSurface_SrcAlpha` liefert dort 0, und die neue Flaeche
        // bekommt `SDL_BLENDMODE_NONE`. Gemessen an `transform.invert`.
        let mut o = Surface::new((2, 2));
        o.set_alpha(Some(255));
        assert_eq!(o.get_alpha(), Some(255));
        assert_eq!(invert(&o).get_alpha(), None);
        // Bei einem anderen Wert bleibt es stehen.
        o.set_alpha(Some(128));
        assert_eq!(invert(&o).get_alpha(), Some(128));
        // Mit Alphakanal entscheidet der Mischmodus, nicht der Wert.
        let mut m = Surface::with_flags((2, 2), SRCALPHA);
        m.set_alpha(Some(255));
        assert_eq!(invert(&m).get_alpha(), Some(255));
        // Die Schluesselfarbe wandert immer mit.
        let mut k = Surface::new((2, 2));
        k.set_colorkey(Some((1, 2, 3)));
        assert_eq!(
            grayscale(&k).get_colorkey().map(|c| c.as_tuple()),
            Some((1, 2, 3, 255))
        );
    }

    #[test]
    fn smoothscale_backend_wie_pygame() {
        // Gemessen an pygame-ce 2.5.7 unter Windows x86-64. Die anderen
        // Kerntests pruefen nur Werte, die auf beiden Wegen gleich sind -
        // das kurze Umschalten hier stoert sie nicht.
        #[cfg(target_arch = "x86_64")]
        {
            assert_eq!(get_smoothscale_backend(), "SSE2");
            assert!(set_smoothscale_backend("GENERIC").is_ok());
            assert_eq!(get_smoothscale_backend(), "GENERIC");
            assert!(set_smoothscale_backend("SSE2").is_ok());
            assert_eq!(get_smoothscale_backend(), "SSE2");
            assert_eq!(
                set_smoothscale_backend("NEON").unwrap_err(),
                "NEON not supported on this machine"
            );
        }
        // Ohne MMX-Uebersetzung sind MMX und SSE **unbekannt**.
        for name in ["MMX", "SSE", "sse2", "", "QUATSCH"] {
            assert_eq!(
                set_smoothscale_backend(name).unwrap_err(),
                format!("Unknown backend type {name}")
            );
        }
        // Und was abgelehnt wurde, aendert nichts.
        #[cfg(target_arch = "x86_64")]
        assert_eq!(get_smoothscale_backend(), "SSE2");
    }

    /// Eine graue Zeile aus den gegebenen Werten.
    fn grauzeile(werte: &[u8]) -> Surface {
        let mut s = Surface::new((werte.len() as u32, 1));
        for (x, v) in werte.iter().enumerate() {
            s.set_at((x as i32, 0), (*v, *v, *v));
        }
        s
    }

    fn rotkanal(s: &Surface) -> Vec<u8> {
        (0..s.get_width() as i32).map(|x| s.get_at((x, 0)).r).collect()
    }

    #[test]
    fn smoothscale_beide_wege_vergroessern() {
        // Der kleinste Fall, in dem pygames zwei Wege auseinandergehen
        // (gesucht und gemessen an pygame-ce 2.5.7): [1, 254] -> 5.
        // SSE2 rechnet mit 8-Bit-Gewichten, GENERIC mit 16-Bit.
        let s = grauzeile(&[1, 254]);
        assert_eq!(
            rotkanal(&smoothscale_mit(&s, (5, 1), Rechenweg::Sse2)),
            vec![1, 51, 101, 152, 202]
        );
        assert_eq!(
            rotkanal(&smoothscale_mit(&s, (5, 1), Rechenweg::Generic)),
            vec![1, 51, 102, 152, 203]
        );
    }

    #[test]
    fn smoothscale_beide_wege_verkleinern() {
        // Dasselbe beim Verkleinern: [200, 100, 50, 25, 10] -> 3. Die
        // Kehrwerte sind nicht dieselben: GENERIC 2^32 / 109226 = 39321,
        // SSE2 2^30 / 27306 = 39322 - der 14-Bit-Weg teilt durch eine
        // kleinere, staerker abgeschnittene Spanne.
        let s = grauzeile(&[200, 100, 50, 25, 10]);
        assert_eq!(
            rotkanal(&smoothscale_mit(&s, (3, 1), Rechenweg::Sse2)),
            vec![159, 54, 15]
        );
        assert_eq!(
            rotkanal(&smoothscale_mit(&s, (3, 1), Rechenweg::Generic)),
            vec![159, 54, 14]
        );
    }

    #[test]
    fn packus_liest_mit_vorzeichen() {
        // `_mm_packus_epi16`: ueber 0x7FFF ist die Spur negativ -> 0.
        assert_eq!(packus_epi16(255), 255);
        assert_eq!(packus_epi16(256), 255);
        assert_eq!(packus_epi16(0x7FFF), 255);
        assert_eq!(packus_epi16(0x8000), 0);
        assert_eq!(packus_epi16(0xFFFF), 0);
    }

    #[test]
    fn leere_flaechen_stuerzen_nirgends_ab() {
        // pygame liefert fuer alle diese Faelle eine 0x0-Flaeche
        // (gemessen); hier ebenso, statt ueber den Puffer zu laufen.
        let leer = Surface::new((0, 0));
        assert_eq!(invert(&leer).get_size(), (0, 0));
        assert_eq!(laplacian(&leer).get_size(), (0, 0));
        assert_eq!(box_blur(&leer, 2, true).unwrap().get_size(), (0, 0));
        assert_eq!(gaussian_blur(&leer, 2, true).unwrap().get_size(), (0, 0));
        assert_eq!(hsl(&leer, 30.0, 0.0, 0.0).unwrap().get_size(), (0, 0));
        assert_eq!(solid_overlay(&leer, (1, 2, 3), false).get_size(), (0, 0));
        assert_eq!(chop(&leer, Rect::new(0, 0, 1, 1)).get_size(), (0, 0));
        assert_eq!(pixelate(&leer, 2).unwrap().get_size(), (0, 0));
        assert_eq!(rotozoom(&leer, 45.0, 1.0).get_size(), (0, 0));
        assert_eq!(average_surfaces(&[&leer, &leer]).unwrap().get_size(), (0, 0));
        // Eine Flaeche ohne Breite behaelt ihre Hoehe.
        let flach = Surface::new((0, 3));
        assert_eq!(box_blur(&flach, 1, true).unwrap().get_size(), (0, 3));
    }
}
