//! surface.rs
//! ==========
//! Die Zeichenflaeche - Gegenstueck zu `pygame.Surface`.
//!
//! Eine Surface ist ein echter Pixelpuffer im Arbeitsspeicher, **ARGB8888**,
//! zeilenweise. Genau wie in pygame: es gibt kein Fenster, keine Grafikkarte,
//! nichts Verstecktes. Was man hineinzeichnet, steht danach als Zahl drin und
//! laesst sich auslesen.
//!
//! **Gemeinsamer Speicher:** `subsurface` teilt sich den Puffer mit seinem
//! Elternteil - Schreiben ins Kind veraendert die Mutter, wie in pygame.
//! Umgesetzt ist das mit [`Shared`]; die Flaeche merkt sich ihren Ausschnitt
//! darin. Der Ausleih-Vermerk kostet einen Test **pro Aufruf**, nicht pro
//! Pixel.

use std::cell::RefCell;
use std::sync::Arc;

use parking_lot::ReentrantMutex;

use crate::blend::{self, flags, BlendOp};
use crate::color::{Color, IntoColor};
use crate::rect::Rect;

/// Wie sich Flaechen Speicher teilen.
///
/// ## Hier stand `Rc<RefCell<T>>`
///
/// Mit dem Vermerk: soll daraus spaeter `Arc<Mutex<..>>` werden (fuer die
/// Python-Huelle oder Threads), sei das eine Zeile statt vierzig Dateien.
/// Der Vermerk hatte recht und war trotzdem zu optimistisch - es waren
/// knapp dreissig Stellen in dieser Datei und noch einmal so viele in der
/// Huelle, weil dort ein **zweites** `Rc<RefCell<..>>` lag.
///
/// Was es ausgeloest hat: `image_test.test_load_bmp_threads` laedt Bilder
/// in zwanzig Faeden und gibt sie an den Hauptfaden weiter. Gebaut wurden
/// sie, aber beim **Wegraeumen** kam
///
/// ```text
/// RuntimeError: PySurface is unsendable,
///               but is being dropped on another thread
/// ```
///
/// Mal zwei Fehler, mal keiner - je nachdem, welcher Faden gerade
/// aufraeumte. Ein Fehler, der wuerfelt, ist schlimmer als einer, der
/// immer kommt.
///
/// ## Warum `ReentrantLock` und nicht `Mutex`
///
/// Ein `Mutex` **haengt**, wenn derselbe Faden zweimal sperrt; ein
/// `RefCell` bricht an derselben Stelle ab. Ein Abbruch ist eine
/// Fehlermeldung, ein Haenger ist ein Programm, das steht - und diese
/// Datei sperrt an mehreren Stellen verschachtelt (`blit` haelt Quelle
/// und Ziel).
///
/// `parking_lot::ReentrantMutex` laesst denselben Faden wieder hinein und
/// gibt nur `&T` heraus. Zusammen mit dem `RefCell` darin ist das **genau die Semantik
/// von vorher** - samt Abbruch bei zwei gleichzeitigen `borrow_mut` -,
/// nur ueber Fadengrenzen hinweg.
///
/// Der Preis steht im Quelltext: `buf.lock().borrow()` statt
/// `buf.borrow()`, und wo das Ergebnis in einer Bindung landet, zwei
/// Zeilen statt einer - die Sperre muss leben, solange die Leihe lebt.
pub type Shared<T> = Arc<ReentrantMutex<RefCell<T>>>;

/// Der rohe Pixelspeicher - **Bytes im gemeldeten Format der Flaeche**.
///
/// ## Warum das einmal anders war
///
/// Hier stand `Vec<u32>` mit einem ARGB-Wort je Pixel, fuer **jede**
/// Tiefe. Befund 4 des Plans sagte: ein Pixelformat reicht. Nach aussen
/// stimmte damit alles - `get_bitsize`, `get_masks`, `get_pitch`, der
/// Tafelindex -, nur die **Bytes im Speicher** waren bei 8, 16 und 24 Bit
/// andere als bei pygame.
///
/// Das faellt auf, sobald jemand den Speicher selbst sieht, und seit es
/// das Puffer-Protokoll gibt, kann er das: `memoryview(s.get_buffer())`,
/// `surfarray.pixels3d(s)`, `pixelcopy.surface_to_array(...)`. Ein Puffer
/// sagt *ab dieser Adresse liegen Pixel in diesem Format*, und das war bei
/// den schmalen Formaten nicht wahr. Zwanzig Tests haben daran gehangen.
///
/// ## Wie es jetzt liegt
///
/// `bpp` Bytes je Pixel, `pitch` Bytes je Zeile (auf vier aufgerundet),
/// niedrigstes Byte zuerst - genau SDLs Anordnung. Der Wert eines Pixels
/// ist der **abgebildete**: bei einer Palettenflaeche der Index, bei 16
/// Bit das RGB565-Wort, bei 32 Bit das ARGB-Wort. Gerechnet wird
/// weiterhin in ARGB; [`PixelBuf::nach_argb`] und
/// [`PixelBuf::von_argb`] sind die Uebergaenge, und bei ARGB8888 sind
/// beide die Kennabbildung.
///
/// ## Warum `Vec<u32>` und nicht `Vec<u8>`
///
/// Ein `Vec<u8>` darf an einer ungeraden Adresse anfangen.
/// `Surface._pixels_address` wandert nach Python und von dort in numpy,
/// und die Feld-Schnittstelle behauptet `PAI_ALIGNED`. Ein `Vec<u32>` ist
/// immer auf vier Bytes ausgerichtet; gerechnet wird trotzdem in Bytes.
#[derive(Debug, Clone)]
pub struct PixelBuf {
    /// Der Speicher, als Vier-Byte-Woerter gehalten und byteweise benutzt.
    pub data: Vec<u32>,
    /// Breite in Pixeln.
    pub w: u32,
    /// Hoehe in Pixeln.
    pub h: u32,
    /// Bytes je Pixel - 1, 2, 3 oder 4.
    pub bpp: u32,
    /// Bytes je Zeile, auf vier aufgerundet.
    pub pitch: u32,
    /// In welchem Format die Bytes hier liegen.
    ///
    /// **Der Puffer kennt sein Format selbst**, und das ist kein
    /// Beiwerk: `blit_roh` bekommt zwei Puffer und keine Flaeche, und es
    /// muss zwischen ihnen umrechnen koennen. Eine Teilflaeche teilt den
    /// Puffer und damit das Format - so wie in pygame auch.
    pub format: Format,
    /// Die Farbtafel, wenn es eine Palettenflaeche ist.
    pub tafel: Option<Vec<Color>>,
}

impl PixelBuf {
    /// Ein neuer Puffer, alles auf null.
    pub fn neu(w: u32, h: u32, format: Format) -> PixelBuf {
        let bpp = format.bytesize().max(1);
        let pitch = (w * bpp + 3) & !3;
        let worte = (pitch as usize * h as usize).div_ceil(4);
        PixelBuf {
            data: vec![0u32; worte],
            w,
            h,
            bpp,
            pitch,
            format,
            tafel: None,
        }
    }

    /// Der Byteversatz einer Stelle im Puffer.
    #[inline]
    pub fn versatz(&self, x: usize, y: usize) -> usize {
        y * self.pitch as usize + x * self.bpp as usize
    }

    /// Der abgebildete Wert an einem Byteversatz.
    #[inline]
    pub fn lies(&self, off: usize) -> u32 {
        if self.bpp == 4 {
            return self.data[off >> 2];
        }
        let b: &[u8] = bytemuck::cast_slice(&self.data);
        match self.bpp {
            1 => u32::from(b[off]),
            2 => u32::from(u16::from_le_bytes([b[off], b[off + 1]])),
            _ => u32::from_le_bytes([b[off], b[off + 1], b[off + 2], 0]),
        }
    }

    /// Legt einen abgebildeten Wert an einem Byteversatz ab.
    #[inline]
    pub fn setz(&mut self, off: usize, v: u32) {
        if self.bpp == 4 {
            self.data[off >> 2] = v;
            return;
        }
        let bpp = self.bpp;
        let b: &mut [u8] = bytemuck::cast_slice_mut(&mut self.data);
        match bpp {
            1 => b[off] = v as u8,
            2 => b[off..off + 2].copy_from_slice(&(v as u16).to_le_bytes()),
            _ => b[off..off + 3].copy_from_slice(&v.to_le_bytes()[..3]),
        }
    }

    /// Vom Speicherwort zum ARGB-Wort.
    #[inline]
    pub fn nach_argb(&self, v: u32) -> u32 {
        if let Some(t) = &self.tafel {
            return t
                .get(v as usize)
                .copied()
                .unwrap_or(Color::rgba(0, 0, 0, 255))
                .to_argb();
        }
        if self.format == Format::ARGB8888 || self.format == Format::XRGB8888 {
            return v;
        }
        self.format.entpacken(v)
    }

    /// Vom ARGB-Wort zum Speicherwort.
    #[inline]
    pub fn von_argb(&self, argb: u32) -> u32 {
        if let Some(t) = &self.tafel {
            return naechster_index(t, argb);
        }
        if self.format == Format::ARGB8888 {
            return argb;
        }
        if self.format == Format::XRGB8888 {
            return argb & 0x00FF_FFFF;
        }
        self.format.packen(argb)
    }

    /// Stellt den Puffer auf ein anderes Format um - **mit** den Pixeln.
    ///
    /// Gebraucht von [`Surface::set_format`]: dort wird eine frisch
    /// angelegte Flaeche nachtraeglich auf ihre Tiefe gesetzt, und die
    /// Bytebreite aendert sich dabei.
    pub fn umstellen(&mut self, format: Format, tafel: Option<Vec<Color>>) {
        let bpp = format.bytesize().max(1);
        let pitch = (self.w * bpp + 3) & !3;
        if bpp == self.bpp && pitch == self.pitch {
            self.format = format;
            self.tafel = tafel;
            return;
        }
        let mut neu = PixelBuf::neu(self.w, self.h, format);
        neu.tafel = tafel;
        for y in 0..self.h as usize {
            for x in 0..self.w as usize {
                let argb = self.nach_argb(self.lies(self.versatz(x, y)));
                let z = neu.versatz(x, y);
                let v = neu.von_argb(argb);
                neu.setz(z, v);
            }
        }
        *self = neu;
    }
}

/// Die Flaeche hat einen eigenen Alphakanal (`pygame.SRCALPHA`).
pub const SRCALPHA: u32 = 0x0001_0000;

/// Das Pixelformat, **wie es nach aussen gemeldet wird**.
///
/// rustgame **rechnet** ausschliesslich in ARGB8888 - das bleibt so, und
/// jeder Puffer im Programm ist ein `Vec<u32>` in dieser Anordnung. pygame
/// kennt daneben aber Flaechen mit 8, 12, 15, 16 und 24 Bit, und der
/// Unterschied ist von aussen sichtbar:
///
/// * `get_bitsize`, `get_bytesize`, `get_masks`, `get_shifts`,
///   `get_losses`, `get_pitch` melden ihn,
/// * `transform` **prueft** ihn: `transform.invert(s32, s16)` gibt
///   `ValueError: Source and destination surfaces need the same format.`,
///   `smoothscale` auf einer Flaeche unter 24 Bit
///   `ValueError: Only 24-bit or 32-bit surfaces can be smoothly scaled`,
///   und die Weichzeichner auf einer Palettenflaeche
///   `ValueError: Indexed surfaces cannot be blurred.` (alles gemessen),
/// * und die **Genauigkeit** haengt daran: `Surface((4,4), depth=16)`,
///   mit `(76,76,76)` gefuellt, liest sich in pygame als `(74,76,74)`
///   zurueck - gemessen.
///
/// Deshalb traegt jede Flaeche das Format als **Angabe** mit, ohne dass
/// sich am Speicher etwas aendert. Ein Pixel wird beim Schreiben auf die
/// Genauigkeit des Formats gebracht ([`Format::quantisieren`]) und bleibt
/// danach ARGB8888 - also laeuft jeder Blit, jeder Zeichenweg und jede
/// Umformung unveraendert weiter.
///
/// Die Standardmasken sind gemessen, nicht hergeleitet:
///
/// ```text
/// Tiefe  bytesize  masks (R/G/B/A)                     shifts        losses
///    8      1      0/0/0/0                             0,0,0,0       8,8,8,8
///   12      2      f00/f0/f/0                          8,4,0,0       4,4,4,8
///   15      2      7c00/3e0/1f/0                       10,5,0,0      3,3,3,8
///   16      2      f800/7e0/1f/0                       11,5,0,0      3,2,3,8
///   24      3      ff0000/ff00/ff/0                    16,8,0,0      0,0,0,8
///   32      4      ff0000/ff00/ff/0                    16,8,0,0      0,0,0,8
///   16+A    2      f00/f0/f/f000                       8,4,0,12      4,4,4,4
///   32+A    4      ff0000/ff00/ff/ff000000             16,8,0,24     0,0,0,0
/// ```
///
/// `SRCALPHA` gibt es also nur bei 16 und 32 Bit - bei 8, 12, 15 und 24
/// meldet pygame `ValueError: no standard masks exist for given bitdepth
/// with alpha`. Und `shifts`/`losses` sind aus den Masken **ableitbar**
/// (Nullen von rechts, bzw. `8 - Anzahl gesetzter Bits`), weshalb sie hier
/// nicht noch einmal abgelegt werden.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Format {
    /// Bits je Pixel, wie pygame sie meldet: 8, 12, 15, 16, 24 oder 32.
    pub bitsize: u32,
    /// Die Bitmasken `(R, G, B, A)`.
    pub masks: (u32, u32, u32, u32),
}

/// Rastet Farben auf das, was eine Flaeche wirklich speichern kann.
///
/// Siehe [`Surface::raster`] - der Typ existiert nur, damit eine
/// Schleife mit ausgeliehenem Puffer das trotzdem kann.
pub struct Raster {
    palette: Option<Vec<Color>>,
    format: Format,
}

impl Raster {
    #[inline]
    /// Rechnet ein ARGB-Wort in den **gemeldeten** Wert dieses Formats
    /// um (Tafelindex, RGB565-Wort, ARGB-Wort).
    pub fn an(&self, argb: u32) -> u32 {
        if let Some(tafel) = &self.palette {
            return naechster_eintrag(tafel, argb);
        }
        self.format.quantisieren(argb)
    }
}

impl Format {
    /// Das Format, in dem rustgame wirklich rechnet.
    pub const ARGB8888: Format = Format {
        bitsize: 32,
        masks: (0x00FF_0000, 0x0000_FF00, 0x0000_00FF, 0xFF00_0000),
    };
    /// Dasselbe ohne Alphakanal - die vierte Maske ist null.
    pub const XRGB8888: Format = Format {
        bitsize: 32,
        masks: (0x00FF_0000, 0x0000_FF00, 0x0000_00FF, 0),
    };

    /// Das Standardformat zu einer Tiefe, wie pygame es waehlt.
    ///
    /// Die Fehlermeldungen sind wortgleich zu pygame gemessen.
    /// **Die Reihenfolge ist gemessen.** `Surface((4,4), SRCALPHA, 7)` gibt
    /// in pygame `no standard masks exist for given bitdepth with alpha`,
    /// **nicht** `nonstandard bit depth given` - obwohl 7 auch keine
    /// gueltige Tiefe ist. pygame prueft also zuerst "mit Alphakanal, und
    /// die Tiefe ist weder 16 noch 32". Hier fing der `_ =>`-Zweig die
    /// Tiefen ausserhalb der Liste vorher ab und gab den falschen Satz.
    /// Vier Nullen heissen **"nimm die Vorgabe"**, nicht "keine Kanaele".
    ///
    /// Gemessen: `Surface((4,4), 0, 32, (0, 0, 0, 0)).get_masks()` gibt
    /// `(16711680, 65280, 255, 0)` - also die Standardmasken fuer 32 Bit
    /// ohne Alphakanal. Hier kamen vier Nullen heraus, und eine Flaeche
    /// ohne jede Farbmaske ist keine Flaeche mehr: `map_rgb` gibt dann
    /// fuer jede Farbe dieselbe Null.
    ///
    /// Der Fall sieht abwegig aus und steht so in `surface_test.test_masks`.
    pub fn aus_masken(bitsize: u32, masks: (u32, u32, u32, u32)) -> Result<Format, &'static str> {
        if masks == (0, 0, 0, 0) {
            return Format::standard(bitsize, false);
        }
        // **Drei Regeln, alle an echtem pygame abgelesen.** Wird eine
        // verletzt, gibt es `ValueError: Invalid mask values`.
        //
        // ```text
        // 32 (0xFF000000, 0xFF0000, 0xFF00, 0)  geht
        // 32 (0x7F0000,   0xFF00,   0xFF,   0)  Fehler - nur 7 Bit
        // 24 (0x7F0000,   0xFF00,   0xFF,   0)  Fehler - dasselbe
        // 32 (0x6F0000,   0xFF00,   0xFF,   0)  Fehler - Luecke in der Maske
        // 16 (0xF800,     0x07E0,   0x001F, 0)  geht   - RGB565
        // 16 (0x7F00,     0xFF,     0xF,    0)  Fehler - 0xFF und 0xF ueberlappen
        // ```
        //
        // Daraus:
        //
        // 1. Jede Maske ist ein **zusammenhaengender** Bitblock.
        // 2. Die Masken **ueberlappen** einander nicht.
        // 3. Ab 24 Bit hat jede genutzte Maske genau **acht** Bits -
        //    pygames Quelltext sagt dazu "For 24 and 32 bit surfaces
        //    Pygame assumes no losses", und `surface_test.test_masks`
        //    steht genau darunter.
        let einzeln = [masks.0, masks.1, masks.2, masks.3];
        let mut belegt: u32 = 0;
        for m in einzeln {
            if m == 0 {
                continue;
            }
            // Zusammenhaengend heisst: nach dem Wegschieben der Nullen
            // rechts ist `m + 1` eine Zweierpotenz.
            let geschoben = m >> m.trailing_zeros();
            if (geschoben & (geschoben + 1)) != 0 {
                return Err("Invalid mask values");
            }
            if belegt & m != 0 {
                return Err("Invalid mask values");
            }
            belegt |= m;
            if bitsize >= 24 && m.count_ones() != 8 {
                return Err("Invalid mask values");
            }
        }
        Ok(Format { bitsize, masks })
    }

    /// Das Format, das pygame fuer diese Tiefe nimmt.
    ///
    /// `Err`, wenn es die Tiefe nicht gibt - pygame sagt dort
    /// `unsupported bit depth for surface`.
    pub fn standard(bitsize: u32, mit_alpha: bool) -> Result<Format, &'static str> {
        if mit_alpha && bitsize != 16 && bitsize != 32 {
            return Err("no standard masks exist for given bitdepth with alpha");
        }
        let masks = match (bitsize, mit_alpha) {
            (8, false) => (0, 0, 0, 0),
            (12, false) => (0x0F00, 0x00F0, 0x000F, 0),
            (15, false) => (0x7C00, 0x03E0, 0x001F, 0),
            (16, false) => (0xF800, 0x07E0, 0x001F, 0),
            (24, false) => (0x00FF_0000, 0x0000_FF00, 0x0000_00FF, 0),
            (32, false) => return Ok(Format::XRGB8888),
            // Mit Alphakanal gibt es nur zwei: ARGB4444 und ARGB8888.
            (16, true) => (0x0F00, 0x00F0, 0x000F, 0xF000),
            (32, true) => return Ok(Format::ARGB8888),
            _ => return Err("nonstandard bit depth given"),
        };
        Ok(Format { bitsize, masks })
    }

    /// Bytes je Pixel. 12 und 15 Bit belegen zwei - deshalb aufgerundet.
    pub fn bytesize(&self) -> u32 {
        self.bitsize.div_ceil(8)
    }

    /// Wie weit jeder Kanal nach links geschoben liegt - die Nullen rechts
    /// in der Maske. Eine leere Maske ergibt `0`, nicht 32.
    pub fn shifts(&self) -> (u32, u32, u32, u32) {
        fn s(m: u32) -> u32 {
            if m == 0 {
                0
            } else {
                m.trailing_zeros()
            }
        }
        let (r, g, b, a) = self.masks;
        (s(r), s(g), s(b), s(a))
    }

    /// Wie viele Bit je Kanal fehlen: acht minus die gesetzten Bits.
    pub fn losses(&self) -> (u32, u32, u32, u32) {
        fn l(m: u32) -> u32 {
            8 - m.count_ones()
        }
        let (r, g, b, a) = self.masks;
        (l(r), l(g), l(b), l(a))
    }

    /// Gehen beim Ablegen einer Farbe ueberhaupt Bits verloren?
    ///
    /// Bei 24 und 32 Bit nicht (das fehlende Alpha faengt
    /// [`Surface::abbilden`] ohnehin ab) - dann bleibt der schnelle Weg
    /// stehen, und an den geprueften Pfaden aendert sich nichts.
    #[inline]
    pub fn verlustfrei(&self) -> bool {
        let (lr, lg, lb, la) = self.losses();
        lr == 0 && lg == 0 && lb == 0 && (la == 0 || self.masks.3 == 0)
    }

    /// Eine Farbe auf die Genauigkeit dieses Formats bringen.
    ///
    /// Zusammendruecken und wieder aufziehen - beides wie SDL:
    /// `v >> loss` hinein, `(v * 255) / ((1 << (8 - loss)) - 1)` heraus
    /// (`SDL_expand_byte`). Gemessen: `(76,76,76)` auf einer 16-Bit-Flaeche
    /// liest sich als `(74,76,74)` - mit der naheliegenden Rechnung
    /// `(v >> loss) << loss` kaeme `(72,76,72)` heraus, was nur fuer
    /// `average_color` stimmt, nicht fuer `get_at`.
    #[inline]
    pub fn quantisieren(&self, argb: u32) -> u32 {
        let (lr, lg, lb, la) = self.losses();
        let a = if self.masks.3 == 0 {
            (argb >> 24) & 0xFF
        } else {
            rundlauf((argb >> 24) & 0xFF, la)
        };
        (a << 24)
            | (rundlauf((argb >> 16) & 0xFF, lr) << 16)
            | (rundlauf((argb >> 8) & 0xFF, lg) << 8)
            | rundlauf(argb & 0xFF, lb)
    }

    /// Eine ARGB8888-Farbe in die Bits dieses Formats packen.
    ///
    /// Gegenstueck zu `SDL_MapRGBA`: jeder Kanal verliert seine unteren
    /// `loss` Bit und wandert an seinen Platz. Gemessen:
    /// `Surface((4,4), depth=16).map_rgb((255,0,0))` ist `0xf800`.
    pub fn packen(&self, argb: u32) -> u32 {
        let (lr, lg, lb, la) = self.losses();
        let (sr, sg, sb, sa) = self.shifts();
        let (mr, mg, mb, ma) = self.masks;
        let mut v = (((argb >> 16) & 0xFF) >> lr) << sr;
        v |= (((argb >> 8) & 0xFF) >> lg) << sg;
        v |= ((argb & 0xFF) >> lb) << sb;
        if ma != 0 {
            v |= (((argb >> 24) & 0xFF) >> la) << sa;
        }
        v & (mr | mg | mb | ma)
    }

    /// Der Rueckweg von [`packen`](Self::packen).
    ///
    /// Ohne Alphakanal kommt `255` heraus - `SDL_GetRGB` setzt das Alpha
    /// dann auf undurchsichtig.
    pub fn entpacken(&self, v: u32) -> u32 {
        let (lr, lg, lb, la) = self.losses();
        let (sr, sg, sb, sa) = self.shifts();
        let (mr, mg, mb, ma) = self.masks;
        let r = ausdehnen((v & mr) >> sr, lr);
        let g = ausdehnen((v & mg) >> sg, lg);
        let b = ausdehnen((v & mb) >> sb, lb);
        let a = if ma == 0 {
            255
        } else {
            ausdehnen((v & ma) >> sa, la)
        };
        (a << 24) | (r << 16) | (g << 8) | b
    }
}

/// Ein Kanalwert hinein und wieder heraus - der Verlust bleibt sichtbar.
#[inline]
fn rundlauf(v: u32, loss: u32) -> u32 {
    if loss == 0 {
        v
    } else {
        ausdehnen(v >> loss, loss)
    }
}

/// `SDL_expand_byte`: aus `8 - loss` Bit wieder acht machen.
///
/// Nicht `v << loss` - das waere um bis zu `2^loss - 1` zu dunkel und
/// wuerde aus Weiss kein Weiss mehr machen.
#[inline]
fn ausdehnen(v: u32, loss: u32) -> u32 {
    if loss == 0 {
        return v;
    }
    let max = (1u32 << (8 - loss)) - 1;
    if max == 0 {
        // Acht Bit Verlust: es gibt nur einen Wert, und das ist Null.
        return 0;
    }
    (v * 255) / max
}

/// Die Farbtafel, die pygame einer frischen 8-Bit-Flaeche mitgibt.
///
/// **Nicht** das naheliegende R3G3B2. Uebertragen aus pygames
/// `src_c/palette.h` (`default_palette_colors`, die SDL-1.2-Tafel; SDL 2
/// legt eine rein weisse an, deshalb setzt pygame sie ueberhaupt) und
/// Eintrag fuer Eintrag gegen `Surface((4,4), depth=8).get_palette()`
/// nachgemessen - alle 256 stimmen:
///
/// ```text
///   Index = r*32 + g*4 + b        r, b: zwei Bit    g: drei Bit
///   r, b in {0, 85, 170, 255}
///   g    in {0, 36, 73, 109, 146, 182, 219, 255}
/// ```
///
/// Damit gibt es nur **128 verschiedene** Farben; die Datei fuehrt sie
/// zweimal auf, Eintrag 128 faengt wieder bei Schwarz an. Deshalb das
/// `& 127`.
///
/// Hier stand zuerst R3G3B2 (Rot in acht Stufen, `Rmask 0xE0`). Das ist
/// die Tafel, die man **erwarten** wuerde, wenn man die Bits zaehlt, und
/// sie ist in 216 von 256 Eintraegen falsch. Aufgefallen ist es an einem
/// einzigen Pixel in `draw_test.test_aafunctions_depth_segfault`
/// (Tiefe 8, Stelle (192,252): 219 statt 255) - der Rest der Datei war
/// gruen. Ein Beispiel dafuer, wie weit eine geratene Tabelle kommt,
/// bevor sie auffliegt.
pub fn palette_vorgabe() -> Vec<Color> {
    /// Zwei Bit auf acht: `0b10` wird `0b10101010` = 170.
    fn zwei(v: u32) -> u8 {
        ((v << 6) | (v << 4) | (v << 2) | v) as u8
    }
    /// Drei Bit auf acht: `0b010` wird `0b01001001` = 73.
    fn drei(v: u32) -> u8 {
        ((v << 5) | (v << 2) | (v >> 1)) as u8
    }
    (0..256u32)
        .map(|i| {
            let k = i & 127;
            Color::rgba(zwei((k >> 5) & 3), drei((k >> 2) & 7), zwei(k & 3), 255)
        })
        .collect()
}

/// Der Tafeleintrag, der einer Farbe am naechsten liegt.
///
/// Abstand im RGB-Raum, ohne Wurzel (die Reihenfolge aendert sie nicht)
/// und ohne Alpha - eine Palettenflaeche hat keines. Gegenstueck zu SDLs
/// `SDL_MapRGB` auf einem Palettenformat, das genauso den kleinsten
/// quadratischen Abstand sucht und bei Gleichstand den **ersten** Eintrag
/// nimmt.
pub fn naechster_eintrag(tafel: &[Color], argb: u32) -> u32 {
    if tafel.is_empty() {
        return argb;
    }
    let (r, g, b) = (
        ((argb >> 16) & 0xFF) as i32,
        ((argb >> 8) & 0xFF) as i32,
        (argb & 0xFF) as i32,
    );
    let mut beste = 0usize;
    let mut bester = i32::MAX;
    for (i, c) in tafel.iter().enumerate() {
        let dr = c.r as i32 - r;
        let dg = c.g as i32 - g;
        let db = c.b as i32 - b;
        let d = dr * dr + dg * dg + db * db;
        if d < bester {
            bester = d;
            beste = i;
            if d == 0 {
                break;
            }
        }
    }
    let c = tafel[beste];
    0xFF00_0000 | ((c.r as u32) << 16) | ((c.g as u32) << 8) | c.b as u32
}

/// Der Index des Tafeleintrags, der einer Farbe am naechsten liegt.
pub fn naechster_index(tafel: &[Color], argb: u32) -> u32 {
    if tafel.is_empty() {
        return 0;
    }
    let ziel = naechster_eintrag(tafel, argb);
    tafel
        .iter()
        .position(|c| {
            0xFF00_0000 | ((c.r as u32) << 16) | ((c.g as u32) << 8) | c.b as u32 == ziel
        })
        .unwrap_or(0) as u32
}

/// Ein Eintrag fuer [`Surface::blits`]: Quelle, Ziel, Ausschnitt, Mischart.
///
/// Als eigener Name, weil das Viertupel sonst in jeder Signatur steht und
/// niemand mehr sieht, was die Zahlen bedeuten.
pub type BlitAuftrag<'a> = (&'a Surface, (i32, i32), Option<Rect>, u32);

/// Eine Zeichenflaeche.
#[derive(Debug, Clone)]
pub struct Surface {
    buf: Shared<PixelBuf>,
    /// Wo diese Flaeche im Puffer liegt (bei `subsurface` verschoben).
    off: (u32, u32),
    size: (u32, u32),
    flags: u32,
    /// Flaechenweites Alpha (`set_alpha`). `None` = aus.
    alpha: Option<u8>,
    /// Der Mischmodus der Flaeche - `SDL_BLENDMODE_BLEND` (`true`) oder
    /// `SDL_BLENDMODE_NONE` (`false`).
    ///
    /// In pygame ist das eine **eigene** Eigenschaft neben dem
    /// Flaechen-Alpha, und sie steuert die halbe Auswahlkette des Blits
    /// (`surface.c:pgSurface_Blit`, `_PgSurface_SrcAlpha`):
    ///
    /// * eine Flaeche mit `SRCALPHA` bekommt ihn bei der Geburt,
    /// * `set_alpha(wert)` setzt ihn - **auch bei `wert == 255`**,
    /// * `set_alpha(None)` nimmt ihn wieder weg, auch einer Flaeche mit
    ///   Alphakanal (`surf_set_alpha` ruft dort `SDL_BLENDMODE_NONE`).
    ///
    /// Der letzte Punkt ist der Grund, warum das nicht aus
    /// `alpha.is_some()` ableitbar ist: eine `SRCALPHA`-Flaeche nach
    /// `set_alpha(None)` wird **kopiert** statt gemischt.
    mischen: bool,
    /// Durchsichtige Farbe (`set_colorkey`), bereits als ARGB gepackt.
    colorkey: Option<u32>,
    /// Nur hier hinein wird gezeichnet (`set_clip`), relativ zur Flaeche.
    clip: Rect,
    /// Der Elternteil, falls dies eine `subsurface` ist.
    parent: Option<Box<Surface>>,
    /// Das **gemeldete** Pixelformat - siehe [`Format`]. Gerechnet wird
    /// immer in ARGB8888.
    format: Format,
    /// Die Farbtafel einer Palettenflaeche (`depth=8`), sonst `None`.
    ///
    /// Sie wird **nicht** zum Rechnen gebraucht - der Puffer haelt auch
    /// hier fertige ARGB-Werte. pygame gibt sie aber heraus
    /// (`get_palette`), traegt sie durch jede Umformung weiter und laesst
    /// `set_palette` die schon abgelegten Pixel **umfaerben**. Genau das
    /// prueft `transform_test.test_invert__palette`.
    palette: Option<Vec<Color>>,
}

impl Surface {
    /// Neue, undurchsichtige Flaeche (alles schwarz).
    pub fn new(size: (u32, u32)) -> Surface {
        Surface::with_flags(size, 0)
    }

    /// Neue Flaeche; mit [`SRCALPHA`] ist sie vollstaendig durchsichtig,
    /// sonst undurchsichtig schwarz - genau wie in pygame.
    pub fn with_flags(size: (u32, u32), flags: u32) -> Surface {
        let (w, h) = size;
        // Beide Faelle fangen bei null an - und das ist bei einer Flaeche
        // **ohne** Alphakanal kein Versehen, sondern gemessen: pygame
        // laesst das ungenutzte vierte Byte dort auf 0 stehen. Sichtbar
        // wird es nie (jeder Leseweg maskiert es auf 255), aber
        // `scale2x` vergleicht ganze 32-Bit-Woerter und merkt es.
        let fuellung = 0x0000_0000u32;
        Surface {
            buf: Arc::new(ReentrantMutex::new(RefCell::new({
                let mut b = PixelBuf::neu(w, h, Format::ARGB8888);
                b.data.fill(fuellung);
                b
            }))),
            off: (0, 0),
            size: (w, h),
            flags,
            alpha: None,
            // pygame gibt einer Flaeche mit Alphakanal gleich bei der
            // Geburt `SDL_BLENDMODE_BLEND` mit, einer ohne nicht.
            mischen: flags & SRCALPHA != 0,
            colorkey: None,
            clip: Rect::new(0, 0, w as i32, h as i32),
            parent: None,
            format: if flags & SRCALPHA != 0 {
                Format::ARGB8888
            } else {
                Format::XRGB8888
            },
            palette: None,
        }
    }

    /// Neue Flaeche mit einem **anderen** gemeldeten Pixelformat.
    ///
    /// Der Speicher bleibt ARGB8888; das Format sagt nur, wie genau
    /// geschrieben wird und was `get_bitsize` und Geschwister melden.
    /// Siehe [`Format`].
    pub fn with_format(size: (u32, u32), flags: u32, format: Format) -> Surface {
        let mut s = Surface::with_flags(size, flags);
        s.set_format(format);
        s
    }

    /// Eine neue Flaeche, die **wie die Vorlage aussieht** - gleiche Flags,
    /// gleiches Flaechen-Alpha, gleiche Schluesselfarbe, aber eigene Groesse
    /// und leerer Inhalt.
    ///
    /// Gegenstueck zu pygames `newsurf_fromsurf`. Ohne das muesste jede
    /// Funktion, die eine Flaeche erzeugt (`transform`, `copy`, spaeter
    /// `image`), diese drei Eigenschaften von Hand nachziehen - und genau
    /// dort geht so etwas verloren.
    pub fn wie(vorlage: &Surface, size: (u32, u32)) -> Surface {
        let mut s = Surface::with_flags(size, vorlage.flags);
        s.alpha = vorlage.alpha;
        s.mischen = vorlage.mischen;
        s.colorkey = vorlage.colorkey;
        // Das Pixelformat gehoert dazu: pygames `newsurf_fromsurf` legt
        // die neue Flaeche im Format der Vorlage an, und wer das vergisst,
        // bekommt aus `transform.scale` einer 8-Bit-Flaeche eine mit 32
        // Bit. Gemessen: `scale(Surface((4,4), depth=8), (8,8))
        // .get_bitsize()` ist 8. Dasselbe gilt fuer die Farbtafel - genau
        // das war der pygame-Fehler 3463, den `test_scale__palette` hueten
        // soll.
        // **Ueber `set_format`, nicht am Feld vorbei.** Seit der Puffer
        // die Bytes im gemeldeten Format haelt, ist ein `s.format = ...`
        // eine halbe Wahrheit: die Flaeche meldet acht Bit und liegt in
        // vier Bytes. `pixelarray_test.test_make_surface` faellt darauf
        // mit `Color(0, 0, 127) != Color(255, 255, 255)` - der Index 127
        // wurde als Farbe gelesen.
        s.set_palette(vorlage.palette.clone());
        s.set_format(vorlage.format);
        s
    }

    // ----- Auskuenfte ---------------------------------------------------

    /// Die Breite.
    pub fn get_width(&self) -> u32 {
        self.size.0
    }

    /// Die Hoehe.
    pub fn get_height(&self) -> u32 {
        self.size.1
    }

    /// Breite und Hoehe.
    pub fn get_size(&self) -> (u32, u32) {
        self.size
    }

    /// Der Mischmodus der Flaeche - `0` ist `SDL_BLENDMODE_NONE`, `1` ist
    /// `SDL_BLENDMODE_BLEND`. Gegenstueck zu `get_blendmode`.
    ///
    /// Das ist **nicht** aus `get_alpha()` ableitbar, und genau deshalb
    /// gibt es die Auskunft. Gemessen:
    ///
    /// ```text
    ///   frisch ohne SRCALPHA        -> 0
    ///   frisch mit  SRCALPHA        -> 1
    ///   nach set_alpha(128)         -> 1
    ///   nach set_alpha(None)        -> 0   (auch mit Alphakanal!)
    /// ```
    pub fn get_blendmode(&self) -> u32 {
        if self.mischen {
            1
        } else {
            0
        }
    }

    /// Ein Rechteck der Flaechengroesse mit Nachkommastellen.
    ///
    /// Gegenstueck zu `get_frect`. Fuer `get_frect(center=…)` einfach
    /// anhaengen: `surf.get_frect().with_center(p)`.
    pub fn get_frect(&self) -> crate::frect::FRect {
        crate::frect::FRect::new(0.0, 0.0, self.size.0 as f32, self.size.1 as f32)
    }

    /// Ein Rechteck der Flaechengroesse, bei `(0, 0)`.
    ///
    /// Fuer `get_rect(center=…)` einfach anhaengen:
    /// `surf.get_rect().with_center(p)`.
    pub fn get_rect(&self) -> Rect {
        Rect::new(0, 0, self.size.0 as i32, self.size.1 as i32)
    }

    /// Die Flaggen der Flaeche (`SRCALPHA`, `RLEACCEL`, ...).
    pub fn get_flags(&self) -> u32 {
        self.flags
    }

    /// Hat die Flaeche einen eigenen Alphakanal?
    pub fn has_alpha(&self) -> bool {
        self.flags & SRCALPHA != 0
    }

    /// Das flaechenweite Alpha (`set_alpha`) - `None`, wenn aus.
    pub fn get_alpha(&self) -> Option<u8> {
        self.alpha
    }

    /// Flaechenweites Alpha. `None` schaltet es ab.
    ///
    /// Achtung, das ist die Stelle, an der viele Nachbauten falsch liegen:
    /// bei einer Flaeche **mit** Alphakanal wird dieser Wert mit dem
    /// Pixel-Alpha **multipliziert** (gemessen: 128 mal 128 wirkt wie 64).
    ///
    /// Und der Aufruf hat eine zweite, unsichtbare Wirkung: er setzt den
    /// Mischmodus. `surf_set_alpha` in `src_c/surface.c`:
    ///
    /// ```c
    /// if (alpha_obj && alpha_obj != Py_None) {
    ///     PG_SetSurfaceBlendMode(surf, SDL_BLENDMODE_BLEND);
    /// } else {
    ///     PG_SetSurfaceBlendMode(surf, SDL_BLENDMODE_NONE);
    /// }
    /// ```
    ///
    /// Deshalb aendert schon `set_alpha(Some(255))` das Ergebnis eines
    /// Blits, obwohl der Wert selbst nichts abdunkelt - er schickt den
    /// Blit in eine andere Routine.
    pub fn set_alpha(&mut self, alpha: Option<u8>) {
        self.alpha = alpha;
        self.mischen = alpha.is_some();
    }

    /// Die Schluesselfarbe - `None`, wenn keine gesetzt ist.
    pub fn get_colorkey(&self) -> Option<Color> {
        // Abgelegt wird die Schluesselfarbe im Format der Flaeche (auf
        // einer ohne Alphakanal also mit Null im vierten Byte), gemeldet
        // wird sie wie jeder andere Farbwert: undurchsichtig.
        self.colorkey.map(|v| Color::from_argb(self.sichtbar(v)))
    }

    /// Diese Farbe wird beim Blitten uebersprungen.
    pub fn set_colorkey(&mut self, key: Option<impl IntoColor>) {
        // Im Format **dieser** Flaeche ablegen, nicht roh: SDL tut das
        // auch. Wichtig wird es bei `alphablit_colorkey`, das den
        // **ganzen** Pixel gegen den Schluessel haelt - auf einer Flaeche
        // ohne Alphakanal steht dort im vierten Byte eine Null, und ein
        // roh abgelegter Schluessel mit 0xFF wuerde nie treffen.
        self.colorkey = key.map(|k| self.abbilden(k));
    }

    /// Der Bereich, in den gezeichnet wird.
    pub fn get_clip(&self) -> Rect {
        self.clip
    }

    /// Begrenzt alles Zeichnen auf dieses Rechteck. `None` hebt es auf.
    pub fn set_clip(&mut self, rect: Option<Rect>) {
        let ganz = self.get_rect();
        self.clip = match rect {
            None => ganz,
            Some(r) => r.normalized().clip(ganz),
        };
    }

    /// Der Elternteil, falls dies eine `subsurface` ist.
    pub fn get_parent(&self) -> Option<&Surface> {
        self.parent.as_deref()
    }

    /// Versatz gegenueber dem Elternteil.
    pub fn get_offset(&self) -> (i32, i32) {
        match &self.parent {
            None => (0, 0),
            Some(p) => (
                self.off.0 as i32 - p.off.0 as i32,
                self.off.1 as i32 - p.off.1 as i32,
            ),
        }
    }

    /// Versatz gegenueber der urspruenglichen Flaeche.
    pub fn get_abs_offset(&self) -> (i32, i32) {
        (self.off.0 as i32, self.off.1 as i32)
    }

    /// Die urspruengliche Flaeche ganz oben in der Kette.
    ///
    /// Bei einer Teilflaeche einer Teilflaeche ist das **nicht** der
    /// direkte Elternteil, sondern die Wurzel.
    pub fn get_abs_parent(&self) -> &Surface {
        match &self.parent {
            None => self,
            Some(p) => p.get_abs_parent(),
        }
    }

    // ----- Das Pixelformat -------------------------------------------------
    //
    // Hier stand: "Hier gibt es nur **ein** Format, ARGB8888 - Paletten
    // und 16 Bit entfallen." Gerechnet wird weiterhin ausschliesslich in
    // ARGB8888, aber **gemeldet** wird, was in [`Format`] steht - eine
    // Flaeche kann 8, 12, 15, 16, 24 oder 32 Bit angeben und eine
    // Farbtafel tragen. Ohne das scheitern in pygames eigener Testsuite
    // `test_invert`, `test_smoothscale`, `test_hsl_args`,
    // `test_scale__destination`, `test_scale__palette`,
    // `test_invert__palette` und `test_grayscale`.
    //
    // Die Vorgabe ist unveraendert und weiter gemessen:
    //
    // ```text
    //                bitsize bytesize pitch(w=7)  masks                               shifts        losses
    // ohne SRCALPHA    32       4        28       ff0000/ff00/ff/0                    16,8,0,0      0,0,0,8
    // mit  SRCALPHA    32       4        28       ff0000/ff00/ff/ff000000             16,8,0,24     0,0,0,0
    // ```

    /// Das gemeldete Pixelformat - siehe [`Format`].
    pub fn get_format(&self) -> Format {
        self.format
    }

    /// Setzt das gemeldete Pixelformat. Am Speicher aendert sich nichts.
    /// Bringt die Flaeche **samt ihren Pixeln** auf ein anderes Format.
    ///
    /// Der Unterschied zu [`set_format`](Self::set_format): dort wird eine
    /// frische Flaeche eingerichtet, hier eine bemalte umgerechnet. Jeder
    /// Pixel geht ueber ARGB - also mit der alten Farbtafel heraus und mit
    /// der neuen hinein.
    pub fn umformen(&mut self, format: Format, tafel: Option<Vec<Color>>) {
        self.buf.lock().borrow_mut().umstellen(format, tafel.clone());
        self.format = format;
        self.palette = tafel;
    }

    /// Setzt das **gemeldete** Format. Siehe [`Surface::umformen`], wenn
    /// auch die Bytes umziehen sollen.
    pub fn set_format(&mut self, format: Format) {
        self.format = format;
        // **Der Puffer muss mit.** Seit die Bytes im gemeldeten Format
        // liegen, ist ein Formatwechsel ein Speicherwechsel: aus vier
        // Bytes je Pixel werden eins, zwei oder drei, und die Zeilenlaenge
        // aendert sich. `Surface::new` legt erst an und setzt dann das
        // Format - ohne diese Zeile blieben die Bytes vierfach breit.
        let tafel = self.palette.clone();
        self.buf.lock().borrow_mut().umstellen(format, tafel);
    }

    /// Die Farbtafel einer Palettenflaeche, sonst `None`.
    pub fn get_palette(&self) -> Option<&[Color]> {
        self.palette.as_deref()
    }

    /// Setzt die Farbtafel (oder nimmt sie weg) - **ohne** die Pixel
    /// anzufassen.
    ///
    /// Fuer den Bauweg gedacht. Wer eine schon bemalte Flaeche umfaerbt,
    /// will [`palette_umfaerben`](Self::palette_umfaerben).
    pub fn set_palette(&mut self, tafel: Option<Vec<Color>>) {
        self.palette = tafel.clone();
        self.buf.lock().borrow_mut().tafel = tafel;
    }

    /// Setzt die Farbtafel und **faerbt die vorhandenen Pixel mit um**.
    ///
    /// In pygame stehen im Puffer einer Palettenflaeche Indizes, nicht
    /// Farben; `set_palette` aendert deshalb rueckwirkend, wie die schon
    /// abgelegten Pixel aussehen. Gemessen:
    ///
    /// ```text
    /// surf = Surface((2,2), depth=8)          # alles Index 0
    /// surf.set_palette(256 * [(10,20,30)])
    /// surf.get_at((1,0))  ->  (10, 20, 30, 255)
    /// ```
    ///
    /// rustgame legt Farben ab, nicht Indizes (siehe
    /// [`einrasten`](Self::einrasten)). Damit dasselbe herauskommt, wird
    /// hier zu jedem Pixel sein **alter** Index gesucht und die Farbe des
    /// neuen Eintrags eingesetzt. Ohne das bliebe eine frische 8-Bit-
    /// Flaeche nach `set_palette` schwarz, waehrend pygame sie in der
    /// neuen Farbe von Eintrag 0 zeigt.
    pub fn palette_umfaerben(&mut self, neu: Vec<Color>) {
        let Some(alt) = self.palette.clone() else {
            self.set_palette(Some(neu));
            return;
        };
        let (w, h) = self.size;
        let tabelle: Vec<u32> = neu
            .iter()
            .map(|c| 0xFF00_0000 | ((c.r as u32) << 16) | ((c.g as u32) << 8) | c.b as u32)
            .collect();
        self.set_palette(Some(neu));
        if tabelle.is_empty() {
            return;
        }
        let ohne_alpha = !self.has_alpha();
        self.mit_pixeln(|puffer, zeilenbreite, (ox, oy)| {
            for y in 0..h as usize {
                for x in 0..w as usize {
                    let i = (oy + y) * zeilenbreite + ox + x;
                    let index = naechster_index(&alt, puffer[i] | 0xFF00_0000) as usize;
                    let wert = tabelle[index.min(tabelle.len() - 1)];
                    puffer[i] = if ohne_alpha { wert & 0x00FF_FFFF } else { wert };
                }
            }
        });
    }

    /// Bits je Pixel.
    pub fn get_bitsize(&self) -> u32 {
        self.format.bitsize
    }

    /// Bytes je Pixel.
    pub fn get_bytesize(&self) -> u32 {
        self.format.bytesize()
    }

    /// Bytes je Bildzeile - **auf vier Bytes aufgerundet**.
    ///
    /// SDL richtet jede Zeile auf vier Bytes aus
    /// (`pitch = (w * bytesize + 3) & ~3`). Bei 32 Bit faellt das nie auf,
    /// bei 8 schon: gemessen ist `Surface((7,7), depth=8).get_pitch()`
    /// gleich **8**, nicht 7, und bei 16 Bit sind es 16, nicht 14.
    ///
    /// Bei einer Teilflaeche gilt die Zeilenlaenge des **Elternpuffers** -
    /// gemessen: `Surface((8,7), 0, 8).subsurface((0,0,5,7)).get_pitch()`
    /// ist 8, nicht 5 oder 8-aus-5. Hier stand dazu "der Kern gibt den
    /// Elternpuffer nicht her, also steht hier die eigene Breite. Siehe
    /// Bericht." Das stimmte, solange es kein `parent`-Feld gab; jetzt
    /// gibt es eines.
    ///
    /// Daran haengt mehr als eine Zahl: `get_view("0")` und `get_view("1")`
    /// gibt pygame nur heraus, wenn `pitch == breite * bytesize` ist, also
    /// wenn die Zeilen **lueckenlos** aufeinander folgen.
    pub fn get_pitch(&self) -> u32 {
        let breite = match &self.parent {
            None => self.size.0,
            Some(p) => p.get_abs_parent().get_width(),
        };
        (breite * self.format.bytesize() + 3) & !3
    }

    /// Die Bitmasken `(R, G, B, A)`. Ohne Alphakanal ist die vierte null.
    pub fn get_masks(&self) -> (u32, u32, u32, u32) {
        self.format.masks
    }

    /// Wie weit jeder Kanal nach links geschoben liegt.
    pub fn get_shifts(&self) -> (u32, u32, u32, u32) {
        self.format.shifts()
    }

    /// Wie viele Bits je Kanal fehlen. Ohne Alphakanal sind es dort acht.
    pub fn get_losses(&self) -> (u32, u32, u32, u32) {
        self.format.losses()
    }

    /// Eine Farbe in das Pixelformat dieser Flaeche uebersetzen.
    ///
    /// Gegenstueck zu `map_rgb`. Ohne Alphakanal faellt das Alpha weg -
    /// gemessen: `map_rgb((1,2,3,4))` ergibt dort `0x00010203`, nicht
    /// `0x04010203`.
    pub fn map_rgb(&self, color: impl IntoColor) -> u32 {
        self.abbilden(color)
    }

    /// Der Pixelwert in den **Bits dieses Formats** - was pygames
    /// `map_rgb` nach Python gibt.
    ///
    /// Bei 32 Bit ist das derselbe Wert wie aus [`abbilden`](Self::abbilden)
    /// (deshalb hing daran lange nichts), bei 16 Bit nicht: gemessen ist
    /// `Surface((4,4), depth=16).map_rgb((255,0,0))` gleich `0xf800`. Auf
    /// einer Palettenflaeche kommt der **Index** heraus.
    pub fn map_rgb_gepackt(&self, color: impl IntoColor) -> u32 {
        let argb = self.abbilden(color);
        if let Some(tafel) = &self.palette {
            return naechster_index(tafel, argb);
        }
        if self.format == Format::ARGB8888 {
            return argb;
        }
        // **Hier wird das unbenutzte Byte abgeschnitten, nicht in der
        // Huelle.** Dort stand `v & 0x00ffffff` fuer jede Flaeche ohne
        // Alphakanal - richtig fuer XRGB8888, wo oben wirklich nichts
        // steht, und falsch fuer eine Flaeche mit den Masken
        // `(0xFF000000, 0xFF0000, 0xFF00, 0)`: dort liegt **Rot** im
        // obersten Byte, und die Huelle warf es weg.
        //
        // Gemessen an echtem pygame: `map_rgb((255,255,255))` gibt dort
        // `0xFFFFFF00`, bei uns kam `0x00FFFF00` heraus - Weiss ohne Rot.
        // Aufgefallen ist es erst mit `gfxdraw_test.py`, das als einzige
        // Datei solche Masken durchprobiert.
        if self.format == Format::XRGB8888 {
            return argb & 0x00FF_FFFF;
        }
        self.format.packen(argb)
    }

    /// Der Rueckweg von [`map_rgb_gepackt`](Self::map_rgb_gepackt).
    pub fn unmap_rgb_gepackt(&self, v: u32) -> Color {
        if let Some(tafel) = &self.palette {
            return *tafel.get(v as usize).unwrap_or(&Color::rgba(0, 0, 0, 255));
        }
        if self.format == Format::ARGB8888 || self.format == Format::XRGB8888 {
            return Color::from_argb(self.sichtbar(v));
        }
        Color::from_argb(self.format.entpacken(v))
    }

    /// Der Rueckweg von [`map_rgb`](Self::map_rgb).
    pub fn unmap_rgb(&self, v: u32) -> Color {
        Color::from_argb(self.sichtbar(v))
    }

    /// Der rohe Pixelwert an dieser Stelle - **ohne** die Maskierung, die
    /// [`get_at`](Self::get_at) vornimmt.
    ///
    /// Genau hier wird sichtbar, dass das vierte Byte einer Flaeche ohne
    /// Alphakanal echter Zustand ist: gemessen liefert
    /// `set_at((0,0), (9,8,7,6))` gefolgt von `get_at_mapped((0,0))` in
    /// pygame `0x00090807` - die 6 ist weg.
    /// **Immer der Rohwert, auch bei 16 Bit oder einer Palette.** Wer den
    /// Wert im gemeldeten Format braucht - was pygames `get_at_mapped`
    /// nach Python gibt -, nimmt
    /// [`map_rgb_gepackt`](Self::map_rgb_gepackt) dazu.
    ///
    /// Hier stand kurzzeitig das Packen mit drin, und das war stiller
    /// Schaden: `transform::lies_roh`, `draw::flood_fill` und
    /// `average_surfaces` lesen ueber diesen Weg den Speicher. Auf einer
    /// Palettenflaeche bekam `invert` damit den **Index** statt der Farbe
    /// und drehte ihn um - aus Blau wurde Weiss statt Gelb. Kein Test im
    /// Kern hat es gemerkt, weil dort keine Palettenflaeche vorkommt.
    pub fn get_at_mapped(&self, pos: (i32, i32)) -> u32 {
        match self.index(pos) {
            None => 0,
            Some(i) => {
                let sperre_buf = self.buf.lock();
                let buf = sperre_buf.borrow();
                buf.nach_argb(buf.lies(i))
            }
        }
    }

    /// Teilen zwei Flaechen denselben Pixelspeicher?
    ///
    /// Wahr fuer eine Flaeche und jede ihrer Teilflaechen, und fuer zwei
    /// Teilflaechen derselben Flaeche - auch wenn sie sich nicht
    /// ueberschneiden. Genau diese Auskunft brauchen pygames
    /// Weichzeichner, die ein Ziel mit gemeinsamem Speicher ablehnen.
    pub fn teilt_speicher(&self, other: &Surface) -> bool {
        Arc::ptr_eq(&self.buf, &other.buf)
    }

    /// Eine Kopie, deren Farben mit ihrem Alpha vormultipliziert sind.
    ///
    /// Braucht einen Alphakanal - pygame wirft sonst einen `ValueError`
    /// mit genau diesem Wortlaut. Die Rechnung ist `((c + 1) * a) >> 8`,
    /// exhaustiv gegen pygame geprueft (alle 65.536 Paare, 0 Abweichungen).
    pub fn premul_alpha(&self) -> Result<Surface, &'static str> {
        if !self.has_alpha() {
            return Err("source surface to be alpha pre-multiplied must have alpha channel");
        }
        let mut neu = self.copy();
        neu.mit_pixeln(|daten, breite, (ox, oy)| {
            for y in 0..self.size.1 as usize {
                for x in 0..self.size.0 as usize {
                    let i = (oy + y) * breite + ox + x;
                    let v = daten[i];
                    let a = (v >> 24) & 0xFF;
                    let f = |c: u32| (((c & 0xFF) + 1) * a) >> 8;
                    daten[i] = (a << 24) | (f(v >> 16) << 16) | (f(v >> 8) << 8) | f(v);
                }
            }
        });
        Ok(neu)
    }

    /// Der schnelle Mehrfach-Blit - `Surface.fblits`.
    ///
    /// Anders als [`blits`](Self::blits) kennt er **keinen Ausschnitt**
    /// und gibt **nichts** zurueck (pygame liefert dort `None`); dafuer
    /// nimmt er das Ziel mit Nachkommastellen entgegen, die beim
    /// Zeichnen abgeschnitten werden - gemessen landet `(1.5, 1.5)` auf
    /// `(1, 1)`.
    ///
    /// Dass er keinen Ausschnitt kennt, hat eine Nebenwirkung, die man ihm
    /// nicht ansieht: `surf_fblits` uebergibt `srcrect` als **`NULL`**
    /// (`surface.c:2572`), und damit fehlt eine der drei Bedingungen, unter
    /// denen pygame den ueberlappenden Selbstblit selbst rechnet. `fblits`
    /// liefert dort also ein anderes Pixel als `blit` - nachgemessen, siehe
    /// `Blitweg::waehlen`.
    pub fn fblits(&mut self, folge: &[(&Surface, (f32, f32))], flags: u32) {
        for (q, ziel) in folge {
            self.fblit(q, (ziel.0 as i32, ziel.1 as i32), flags);
        }
    }

    /// **Ein** Eintrag aus [`fblits`](Self::fblits).
    ///
    /// Gibt es, weil die Huelle ihre Liste selbst durchlaeuft (jede der drei
    /// Methoden prueft ihre Argumente anders) und dabei nicht in `blit_ex`
    /// landen darf: das leere `srcrect` gehoert zu `fblits`.
    pub fn fblit(&mut self, src: &Surface, dest: (i32, i32), flags: u32) -> Rect {
        self.blit_intern(src, dest, None, flags, false)
    }

    /// Mehrere Blits in einem Aufruf - `Surface.blits`.
    ///
    /// Gibt die beschriebenen Rechtecke zurueck, eines je Eintrag.
    pub fn blits(&mut self, folge: &[BlitAuftrag<'_>]) -> Vec<Rect> {
        folge
            .iter()
            .map(|(q, ziel, bereich, flags)| self.blit_ex(q, *ziel, *bereich, *flags))
            .collect()
    }

    // ----- Einzelne Pixel ------------------------------------------------

    /// Farbe eines Pixels. Ausserhalb der Flaeche kommt Schwarz zurueck.
    pub fn get_at(&self, pos: (i32, i32)) -> Color {
        match self.index(pos) {
            None => Color::rgba(0, 0, 0, 0),
            Some(i) => {
                let sperre_buf = self.buf.lock();
                let buf = sperre_buf.borrow();
                Color::from_argb(self.sichtbar(buf.nach_argb(buf.lies(i))))
            }
        }
    }

    /// Bildet eine Farbe auf das Format **dieser** Flaeche ab.
    ///
    /// Gegenstueck zu pygames `PG_MapRGBA`. Hat die Flaeche keinen
    /// Alphakanal (`Amask == 0`), faellt das Alpha weg und der Pixel wird
    /// undurchsichtig. Gemessen an echtem pygame - und zwar fuer **jeden**
    /// Schreibweg, nicht nur fuer `fill`:
    ///
    /// ```text
    /// Flaeche ohne SRCALPHA, Farbe (255, 0, 0, 128):
    ///   fill    -> (255, 0, 0, 255)
    ///   set_at  -> (255, 0, 0, 255)
    ///   draw    -> (255, 0, 0, 255)
    ///   blit    -> Alpha 255 (der Wert 128 mischt, wird aber nicht abgelegt)
    /// ```
    ///
    /// Deshalb steht das hier an einer Stelle und nicht verstreut in
    /// `draw`, `fill` und `blit`.
    #[inline]
    pub fn abbilden(&self, color: impl IntoColor) -> u32 {
        let v = color.into_color().to_argb();
        // Bei 24 und 32 Bit geht nichts verloren - dann bleibt der Weg
        // genau der, der geprueft ist. Nur die schmalen Formate (8, 12,
        // 15, 16) verlieren hier Bits, und eine Palettenflaeche rastet
        // auf ihren naechsten Tafeleintrag ein.
        let v = self.einrasten(v);
        if self.has_alpha() {
            v
        } else {
            // `PG_MapRGBA` mit `Amask == 0` laesst das Alpha **fallen**,
            // es schreibt keine 255 hinein. Gemessen am Rohspeicher:
            // `fill((72,197,142))` auf einer Flaeche ohne Alphakanal
            // ergibt die Bytes `8ec54800` - das vierte ist null.
            v & 0x00FF_FFFF
        }
    }

    /// Eine Farbe auf das ab, was in dieser Flaeche darstellbar ist.
    ///
    /// Bei einer Palettenflaeche ist das der **naechste** Eintrag der
    /// Farbtafel, sonst die Genauigkeit des Formats. Gemessen an echtem
    /// pygame:
    ///
    /// ```text
    /// s = Surface((32,32), depth=8)
    /// s.set_palette([red, purple, gold, blue, lightgreen])
    /// s.fill("red"); transform.invert(s).get_at((0,0))  -> lightgreen
    /// ```
    ///
    /// Das Gegenteil von rot ist Cyan, und der naechste Tafeleintrag dazu
    /// ist eben Hellgruen. rustgame legt im Puffer weiterhin fertige
    /// ARGB-Werte ab statt Indizes - das Einrasten hier gibt dieselbe
    /// sichtbare Farbe, ohne dass Blit und Zeichnen etwas davon merken.
    #[inline]
    /// Ein **vorbereiteter** Raster fuer Schleifen, die den Puffer schon
    /// ausgeliehen haben.
    ///
    /// [`einrasten`](Self::einrasten) braucht `&self` und laesst sich
    /// deshalb nicht mitten in einer `buf.borrow_mut()`-Schleife rufen.
    /// Dieser Helfer holt vorher, was gebraucht wird.
    ///
    /// **`None` heisst "nichts zu tun"** - und das ist der Normalfall:
    /// bei 24 und 32 Bit ohne Farbtafel ist das Einrasten die Identitaet.
    /// Der geprüfte Pixelweg (123 Referenzbilder, 0 von 76.800
    /// Pixelbytes) laeuft damit Wort fuer Wort wie vorher; nur 8, 12, 15
    /// und 16 Bit bekommen ueberhaupt eine Schleifenlast.
    pub fn raster(&self) -> Option<Raster> {
        if self.palette.is_none() && self.format.verlustfrei() {
            return None;
        }
        Some(Raster {
            palette: self.palette.clone(),
            format: self.format,
        })
    }

    /// Dieselbe Farbe, nachdem sie durch das Format dieser Flaeche
    /// gelaufen ist - was in RGB565 nicht darstellbar ist, rastet ein.
    pub fn einrasten(&self, argb: u32) -> u32 {
        if let Some(tafel) = &self.palette {
            return naechster_eintrag(tafel, argb);
        }
        if self.format.verlustfrei() {
            return argb;
        }
        self.format.quantisieren(argb)
    }

    /// Der Pixelwert an dieser Stelle **im Format dieser Flaeche** - was
    /// `PixelArray` und `Surface.get_at_mapped` nach Python geben.
    ///
    /// Das ist nicht [`get_at_mapped`](Self::get_at_mapped): der gibt den
    /// Rohwert, wie er im Speicher steht, und der Speicher ist hier immer
    /// ARGB32. Auf einer Palettenflaeche steht dort also eine **Farbe**,
    /// waehrend pygame den **Index** meldet - genau die Verwechslung, an
    /// der `pixelarray_test.py` mit `AssertionError: 255 != 3` haengt.
    ///
    /// Bei 32 Bit sind beide gleich; deshalb faellt der Unterschied nur
    /// bei 8 und 16 Bit auf, und deshalb hat er lange keiner gesehen.
    pub fn gepackt_an(&self, pos: (i32, i32)) -> u32 {
        // **Direkt aus dem Speicher, nicht ueber die Farbe.**
        //
        // Hier stand `map_rgb_gepackt(get_at(pos))` - Index heraus, Farbe
        // daraus, naechster Index dazu. Das ging, solange im Speicher
        // Farben lagen; seit dort der **Index** steht, ist es ein Umweg
        // mit Verlust: pygames Standardtafel fuehrt jede ihrer 128 Farben
        // zweimal, also kam aus 128 eine 0 und aus 200 eine 72.
        //
        // Genau das war die benannte Grenze `pixelarray.tafelindex`. Sie
        // ist damit weg - nicht, weil hier etwas schlauer geworden waere,
        // sondern weil der Speicher jetzt das Richtige enthaelt.
        match self.index(pos) {
            None => 0,
            Some(i) => self.buf.lock().borrow().lies(i),
        }
    }

    /// Legt einen **gepackten** Wert ab, ohne den Weg ueber die Farbe.
    ///
    /// Das Gegenstueck zu [`gepackt_an`](Self::gepackt_an), und aus
    /// demselben Grund noetig: `PixelArray` und `pixelcopy` reichen
    /// Pixelwerte als Zahlen durch, und eine Zahl, die durch eine Farbe
    /// gegangen ist, ist auf einer Palettenflaeche nicht mehr dieselbe.
    pub fn setz_gepackt(&mut self, pos: (i32, i32), v: u32) -> bool {
        match self.index(pos) {
            None => false,
            Some(i) => {
                self.buf.lock().borrow_mut().setz(i, v);
                true
            }
        }
    }

    /// Der Rueckweg: ein gepackter Wert wird zu einem Speicherwort.
    ///
    /// Wer eine blanke Zahl roh in den Speicher schreibt, legt auf einer
    /// Palettenflaeche die 3 als Farbe `(0, 0, 3)` ab statt als dritten
    /// Tafeleintrag.
    pub fn aus_gepackt(&self, v: u32) -> u32 {
        self.abbilden(self.unmap_rgb_gepackt(v))
    }

    /// Wie ein Pixel **von aussen** aussieht.
    ///
    /// Ohne Alphakanal ist er immer undurchsichtig, ganz gleich, was im
    /// ungenutzten vierten Byte steht. Genau das tun `get_at` und
    /// `tobytes` in pygame auch - deshalb zeigt keine der ueblichen
    /// Messungen, dass dort ueberhaupt etwas anderes stehen kann.
    #[inline]
    fn sichtbar(&self, v: u32) -> u32 {
        if self.has_alpha() {
            v
        } else {
            v | 0xFF00_0000
        }
    }

    /// Setzt ein Pixel hart (kein Mischen) - achtet auf das Clip-Rechteck.
    ///
    /// Gibt zurueck, **ob** geschrieben wurde. pygames `set_and_check_rect`
    /// braucht genau diese Auskunft, um das beschriebene Rechteck
    /// mitzuschreiben; ohne sie muss jede Zeichenfunktion den Clip ein
    /// zweites Mal pruefen.
    pub fn set_at(&mut self, pos: (i32, i32), color: impl IntoColor) -> bool {
        if !self.clip.collidepoint(pos) {
            return false;
        }
        let farbe = self.abbilden(color);
        match self.index(pos) {
            None => false,
            Some(i) => {
                let sperre_buf = self.buf.lock();
                let mut buf = sperre_buf.borrow_mut();
                let v = buf.von_argb(farbe);
                buf.setz(i, v);
                true
            }
        }
    }

    /// [`set_at`](Self::set_at) fuer **jedes** Pixel, zeilenweise aus
    /// `[r, g, b, a]` - bei den zwei Standardformaten in einem Zug.
    ///
    /// Gebraucht von `transform`, das ganze Bilder auf einmal ablegt.
    /// `set_at` nimmt je Pixel die Sperre, bildet die Farbe ab und rechnet
    /// sie ins Speicherformat um; gemessen in Runde 18 war das der groesste
    /// Posten von `smoothscale` (11 ms fuer 1280x960 auf ein Viertel,
    /// pygame 0,6 ms). Bei vier Bytes ohne Farbtafel und vollem
    /// Clip-Rechteck ist das, was `set_at` ablegt, `pack(a, r, g, b)` -
    /// ohne Alphakanal mit geloeschtem viertem Byte (`abbilden`), bei
    /// XRGB8888 noch einmal maskiert (`von_argb`). Alles andere geht
    /// wirklich durch `set_at`.
    pub fn setze_alle(&mut self, farben: &[[u8; 4]]) {
        let (w, h) = (self.size.0 as usize, self.size.1 as usize);
        let voll = self.clip == Rect::new(0, 0, self.size.0 as i32, self.size.1 as i32);
        let standard = {
            let sperre_b = self.buf.lock();
            let b = sperre_b.borrow();
            ist_standard32(&b)
        };
        if standard && voll {
            let abbild_maske = if self.has_alpha() {
                0xFFFF_FFFFu32
            } else {
                0x00FF_FFFF
            };
            let (ox, oy) = (self.off.0 as usize, self.off.1 as usize);
            let sperre_buf = self.buf.lock();
            let mut buf = sperre_buf.borrow_mut();
            let format_maske = if buf.format == Format::ARGB8888 {
                0xFFFF_FFFFu32
            } else {
                0x00FF_FFFF
            };
            let maske = abbild_maske & format_maske;
            let zw = (buf.pitch / 4) as usize;
            for y in 0..h {
                let a = (oy + y) * zw + ox;
                for (wort, p) in buf.data[a..a + w].iter_mut().zip(&farben[y * w..(y + 1) * w]) {
                    *wort = blend::pack(p[3], p[0], p[1], p[2]) & maske;
                }
            }
            return;
        }
        for y in 0..h {
            for x in 0..w {
                let p = farben[y * w + x];
                self.set_at((x as i32, y as i32), (p[0], p[1], p[2], p[3]));
            }
        }
    }

    /// Wie [`set_at`](Self::set_at), aber **ohne** Clip-Pruefung.
    ///
    /// Gegenstueck zu pygames `unsafe_set_at`. Fuer heisse Schleifen, die
    /// den Clip bereits selbst eingerechnet haben - duenne Umrisse etwa
    /// zahlen sonst je Pixel eine Rechteckpruefung. Liegt `pos` ausserhalb
    /// der Flaeche, passiert nichts (kein Absturz, kein fremder Speicher).
    #[inline]
    pub fn set_at_ungeprueft(&mut self, pos: (i32, i32), farbe: u32) {
        if let Some(i) = self.index(pos) {
            let sperre_buf = self.buf.lock();
            let mut buf = sperre_buf.borrow_mut();
            let v = buf.von_argb(farbe);
            buf.setz(i, v);
        }
    }

    /// Reicht den **ganzen** Pixelspeicher an eine Schleife weiter: einmal
    /// ausleihen statt einmal je Pixel.
    ///
    /// Die Schleife bekommt den Speicher, die Zeilenbreite des Puffers und
    /// den Versatz dieser Flaeche darin - bei einer `subsurface` ist der
    /// naemlich nicht null. Das ist der Weg fuer `transform` und `image`,
    /// die ganze Bilder auf einmal schreiben; `set_at` je Pixel kostet dort
    /// ein Vielfaches.
    /// Die Stelle des ersten Pixels im Speicher - `Surface._pixels_address`.
    ///
    /// Bei einer Teilflaeche ist es der Anfang **ihres** Bereichs, nicht
    /// der des Elternteils - so wie pygames `surf->pixels` bei einer
    /// Teilflaeche schon versetzt ist.
    ///
    /// **Was dort liegt, ist immer ARGB mit vier Bytes**, auch wenn die
    /// Flaeche 8 oder 16 Bit meldet. Das ist der Unterschied, an dem die
    /// vier `test_array_interface*` haengen: pygames Puffer hat wirklich
    /// die gemeldete Breite. Hier ist die Zahl ehrlich - der Speicher
    /// existiert und liegt dort -, aber sie beschreibt rustgames Ablage,
    /// nicht das gemeldete Format. Siehe die benannte Grenze dazu.
    ///
    /// **Kein `unsafe`:** einen Zeiger zu bilden ist sicher, nur ihm zu
    /// folgen waere es nicht - und das tut hier niemand.
    pub fn pixel_zeiger(&self) -> usize {
        let sperre_buf = self.buf.lock();
        let buf = sperre_buf.borrow();
        let versatz = buf.versatz(self.off.0 as usize, self.off.1 as usize);
        // `wrapping_add` statt `add`, weil `add` bei einem leeren Puffer
        // auf einen Zeiger hinter dem Ende rechnen wuerde.
        (buf.data.as_ptr() as usize).wrapping_add(versatz)
    }

    /// Gibt den ganzen Puffer als **ARGB-Woerter** heraus und nimmt ihn
    /// wieder zurueck.
    ///
    /// Die Aufrufer - Bildladen, `transform`, die Formatkisten - rechnen
    /// in ARGB und schreiben ganze Bilder auf einmal. Seit der Puffer
    /// Bytes im gemeldeten Format haelt, wird hier **umgeschrieben**: erst
    /// in ein Feld aus Woertern, dann der Rueckweg.
    ///
    /// Bei 32 Bit ist das ein Hin- und Herkopieren und sonst nichts; bei
    /// 8, 16 und 24 Bit ist es die Umrechnung, die es vorher gar nicht
    /// gab. Beides geschieht **einmal je Aufruf** und nicht je Pixel -
    /// die Aufrufer sind Massenarbeiten, keine heissen Schleifen.
    pub fn mit_pixeln<R>(&mut self, f: impl FnOnce(&mut [u32], usize, (usize, usize)) -> R) -> R {
        let versatz = (self.off.0 as usize, self.off.1 as usize);
        let (w, h) = {
            let sperre_b = self.buf.lock();
            let b = sperre_b.borrow();
            (b.w as usize, b.h as usize)
        };
        let mut worte = vec![0u32; w * h];
        // **Bei den Standardformaten ist der Hin- und Rueckweg eine Kopie.**
        // `nach_argb` ist dort die Identitaet, `von_argb` bei ARGB8888 auch
        // und bei XRGB8888 eine Maske, und ein Puffer mit vier Bytes je
        // Pixel hat genau `w * h` Woerter. Gemessen in Runde 18: ein
        // `font.render` verbrachte die meiste Zeit in diesen zwei Schleifen.
        // Die Sperre wird weiter **nicht** ueber `f` gehalten.
        let standard = {
            let sperre_b = self.buf.lock();
            let b = sperre_b.borrow();
            ist_standard32(&b) && b.data.len() == w * h
        };
        if standard {
            {
                let sperre_buf = self.buf.lock();
                let buf = sperre_buf.borrow();
                worte.copy_from_slice(&buf.data);
            }
            let r = f(&mut worte, w, versatz);
            let sperre_buf = self.buf.lock();
            let mut buf = sperre_buf.borrow_mut();
            if buf.format == Format::ARGB8888 {
                buf.data.copy_from_slice(&worte);
            } else {
                for (d, s) in buf.data.iter_mut().zip(&worte) {
                    *d = s & 0x00FF_FFFF;
                }
            }
            return r;
        }
        {
            let sperre_buf = self.buf.lock();
            let buf = sperre_buf.borrow();
            for y in 0..h {
                for x in 0..w {
                    worte[y * w + x] = buf.nach_argb(buf.lies(buf.versatz(x, y)));
                }
            }
        }
        let r = f(&mut worte, w, versatz);
        {
            let sperre_buf = self.buf.lock();
            let mut buf = sperre_buf.borrow_mut();
            for y in 0..h {
                for x in 0..w {
                    let v = buf.von_argb(worte[y * w + x]);
                    let off = buf.versatz(x, y);
                    buf.setz(off, v);
                }
            }
        }
        r
    }

    /// Wo liegt dieses Pixel im gemeinsamen Puffer?
    #[inline]
    fn index(&self, pos: (i32, i32)) -> Option<usize> {
        if pos.0 < 0 || pos.1 < 0 || pos.0 >= self.size.0 as i32 || pos.1 >= self.size.1 as i32 {
            return None;
        }
        let sperre_buf = self.buf.lock();
        let buf = sperre_buf.borrow();
        let x = self.off.0 as usize + pos.0 as usize;
        let y = self.off.1 as usize + pos.1 as usize;
        // **Ein Byteversatz, keine Pixelnummer.** Seit der Puffer Bytes im
        // gemeldeten Format haelt, sind die beiden nur bei 32 Bit dasselbe.
        Some(buf.versatz(x, y))
    }

    // ----- Fuellen --------------------------------------------------------

    /// Fuellt die ganze Flaeche (bzw. das Clip-Rechteck).
    pub fn fill(&mut self, color: impl IntoColor) -> Rect {
        let r = self.clip;
        self.fill_rect(color, r)
    }

    /// Fuellt ein Rechteck.
    pub fn fill_rect(&mut self, color: impl IntoColor, rect: Rect) -> Rect {
        self.fill_ex(color, Some(rect), 0)
    }

    /// Fuellt mit einer Mischart (`special_flags`).
    pub fn fill_ex(&mut self, color: impl IntoColor, rect: Option<Rect>, flags: u32) -> Rect {
        let ziel = match rect {
            None => self.clip,
            Some(r) => r.normalized().clip(self.clip),
        };
        if ziel.is_empty() {
            // **Alle vier Werte null**, nicht die Ecke behalten. Gemessen:
            // `s.fill(c, (-30, -30, 5, 5))` gibt `Rect(0, 0, 0, 0)`, nicht
            // `Rect(-30, -30, 0, 0)`.
            //
            // Die Ecke mitzunehmen sieht nach der genaueren Auskunft aus
            // und ist die falsche: gefuellt wurde **nichts**, und ein
            // Rechteck ohne Flaeche hat keine Lage. `surface_test` sagt es
            // im Kommentar sogar - "since we are using negative coords, it
            // should be a zero sized rect".
            return Rect::new(0, 0, 0, 0);
        }
        let farbe = self.abbilden(color);
        // **Ein gemischtes Ergebnis muss einrasten.**
        //
        // `flags == 0` legt die schon abgebildete Farbe ab und braucht
        // nichts weiter; jede Mischart rechnet dagegen einen **neuen**
        // Wert aus, und der muss auf das Format der Flaeche passen.
        // Gemessen an einer 8-Bit-Flaeche: nach `BLEND_ADD` gibt pygame
        // den Tafeleintrag `146`, hier stand `145` - eine Zahl, die in der
        // Tafel gar nicht vorkommt. Fuenf Tests in `surface_test.py`
        // (`test_fill_blend`, `test_blit_blend`, deren `_rgba`-Zwillinge
        // und `test_blanket_alpha`) pruefen das ueber **alle** Tiefen.
        let raster = if flags == 0 { None } else { self.raster() };
        let (ox, oy) = (self.off.0 as usize, self.off.1 as usize);
        let sperre_buf = self.buf.lock();
        let mut buf = sperre_buf.borrow_mut();

        // **Ohne Mischart landet ueberall dasselbe Wort** - die Schleife
        // darunter rechnet es nur fuer jedes Pixel neu aus (`von_argb` der
        // schon abgebildeten Farbe; eingerastet wird bei `flags == 0`
        // nicht). Gemessen in Runde 18: `fill` 200x120 kostete 64,6 us,
        // pygame 4,4 us. Bei vier Bytes je Pixel wird die Zeile jetzt in
        // einem Zug beschrieben.
        if flags == 0 && buf.bpp == 4 {
            let wort = buf.von_argb(farbe);
            let zw = (buf.pitch / 4) as usize;
            for y in ziel.y..ziel.y + ziel.h {
                let a = (oy + y as usize) * zw + ox + ziel.x as usize;
                buf.data[a..a + ziel.w as usize].fill(wort);
            }
            return ziel;
        }

        for y in ziel.y..ziel.y + ziel.h {
            for x in ziel.x..ziel.x + ziel.w {
                let i = buf.versatz(ox + x as usize, oy + y as usize);
                // Gerechnet wird in ARGB, abgelegt im Format der Flaeche.
                let alt = buf.nach_argb(buf.lies(i));
                let neu = match flags {
                    0 => farbe,
                    flags::BLEND_RGB_ADD => blend::RgbAdd::apply(alt, farbe, 255),
                    flags::BLEND_RGB_SUB => blend::RgbSub::apply(alt, farbe, 255),
                    flags::BLEND_RGB_MULT => blend::RgbMult::apply(alt, farbe, 255),
                    flags::BLEND_RGB_MIN => blend::RgbMin::apply(alt, farbe, 255),
                    flags::BLEND_RGB_MAX => blend::RgbMax::apply(alt, farbe, 255),
                    flags::BLEND_RGBA_ADD => blend::RgbaAdd::apply(alt, farbe, 255),
                    flags::BLEND_RGBA_SUB => blend::RgbaSub::apply(alt, farbe, 255),
                    flags::BLEND_RGBA_MULT => blend::RgbaMult::apply(alt, farbe, 255),
                    flags::BLEND_RGBA_MIN => blend::RgbaMin::apply(alt, farbe, 255),
                    flags::BLEND_RGBA_MAX => blend::RgbaMax::apply(alt, farbe, 255),
                    _ => farbe,
                };
                let neu = match &raster {
                    Some(r) => r.an(neu),
                    None => neu,
                };
                let v = buf.von_argb(neu);
                buf.setz(i, v);
            }
        }
        ziel
    }

    // ----- Blitten ---------------------------------------------------------

    /// Zeichnet eine andere Flaeche hierher.
    pub fn blit(&mut self, src: &Surface, dest: (i32, i32)) -> Rect {
        self.blit_ex(src, dest, None, 0)
    }

    /// Wie [`Surface::blit`], aber nur ein Ausschnitt der Quelle.
    pub fn blit_area(&mut self, src: &Surface, dest: (i32, i32), area: Rect) -> Rect {
        self.blit_ex(src, dest, Some(area), 0)
    }

    /// Der vollstaendige Blit mit Ausschnitt und Mischart.
    ///
    /// Gibt das tatsaechlich beschriebene Rechteck zurueck (auf das
    /// Clip-Rechteck beschnitten) - wie in pygame.
    pub fn blit_ex(
        &mut self,
        src: &Surface,
        dest: (i32, i32),
        area: Option<Rect>,
        flags: u32,
    ) -> Rect {
        // `srcrect != NULL` - `surf_blit` und `surf_blits` fuellen es selbst
        // aus, auch bei `area=None`. Nur `fblits` laesst es leer, siehe
        // `Blitweg::waehlen`.
        self.blit_intern(src, dest, area, flags, true)
    }

    /// Der Blit mit dem letzten Schalter, den pygames C-Schnittstelle kennt:
    /// ob ein Quellrechteck mitkommt. Siehe `Blitweg::waehlen`.
    fn blit_intern(
        &mut self,
        src: &Surface,
        dest: (i32, i32),
        area: Option<Rect>,
        flags: u32,
        hat_quellrechteck: bool,
    ) -> Rect {
        // Welcher Teil der Quelle wird ueberhaupt gelesen?
        let roh = match area {
            None => src.get_rect(),
            Some(a) => a.normalized(),
        };
        let quelle = roh.clip(src.get_rect());
        if quelle.is_empty() {
            return Rect::new(dest.0, dest.1, 0, 0);
        }

        // **Ragt der Ausschnitt links oder oben aus der Quelle heraus,
        // wandert das Ziel um denselben Betrag mit.** Das ist SDLs
        // Verhalten in `SDL_UpperBlit` und nicht zu erraten - gemessen:
        //
        // ```text
        // Ziel 16x16, Quelle 8x8, dest=(5,5)
        //   area=(-3,-2,6,6) -> gezeichnet bei (8, 7, 3, 4)
        //   area=( 6, 6,8,8) -> gezeichnet bei (5, 5, 2, 2)
        // ```
        //
        // Rechts und unten wird nur gekuerzt, links und oben verschoben.
        // Der differenzielle Fuzzer hat genau das gefunden; keiner der
        // handgeschriebenen Referenzfaelle kam darauf.
        let dest = (dest.0 + (quelle.x - roh.x), dest.1 + (quelle.y - roh.y));

        // Zielrechteck, auf das Clip-Rechteck beschnitten.
        let ziel_voll = Rect::new(dest.0, dest.1, quelle.w, quelle.h);
        let ziel = ziel_voll.clip(self.clip);
        if ziel.is_empty() {
            // **Der leere Rueckgabewert sieht nicht ueberall gleich aus.**
            // Faellt der Blit ganz weg, meldet pygames eigener Weg
            // (`alphablit_alpha`, also eine Quelle **mit** Alphakanal ohne
            // Schluesselfarbe) negative Koordinaten auf null geklemmt,
            // SDLs Wege dagegen unveraendert. Gemessen, Ziel 48x48,
            // Quelle 6x3:
            //
            // ```text
            //                  blit(.., (18,-5))   blit(.., (-9,18))
            //   undurchsichtig   (18, -5, 0, 0)      (-9, 18, 0, 0)
            //   set_alpha(128)   (18, -5, 0, 0)      -
            //   SRCALPHA         (18,  0, 0, 0)      ( 0, 18, 0, 0)
            // ```
            //
            // Ueberstand nach rechts oder unten wird in keinem Fall
            // geklemmt - nur die negative Seite.
            if src.has_alpha() && src.colorkey.is_none() && src.mischen {
                return Rect::new(dest.0.max(0), dest.1.max(0), 0, 0);
            }
            return Rect::new(dest.0, dest.1, 0, 0);
        }
        // Um wie viel wurde links/oben abgeschnitten?
        let schnitt = (ziel.x - ziel_voll.x, ziel.y - ziel_voll.y);
        let quelle_start = (quelle.x + schnitt.0, quelle.y + schnitt.1);

        // Das Flaechen-Alpha der Quelle gilt fuer alle Pixel.
        let flaechen_alpha = src.alpha.unwrap_or(255);
        let quelle_hat_alpha = src.has_alpha();
        let ziel_hat_alpha = self.has_alpha();

        // Zeichnen Quelle und Ziel in denselben Puffer? Dann darf nur ein
        // einziger Ausleih-Vermerk bestehen - sonst bricht RefCell ab.
        let gleicher_puffer = Arc::ptr_eq(&self.buf, &src.buf);

        let ziel_raster = self.raster();
        let info = BlitInfo {
            raster: ziel_raster,
            weg: Blitweg::waehlen(
                flags,
                src,
                self,
                hat_quellrechteck,
                gleicher_puffer && self.ueberlappt_mit(src, ziel, quelle_start),
            ),
            flaechen_alpha,
            quelle_hat_alpha,
            ziel_hat_alpha,
            // Was die `BLEND_RGBA_*`-Mischarten im Alphakanal der Quelle
            // sehen. Gemessen an echtem pygame, und ueberraschend:
            //
            // | Quelle | Mischmodus | wirksames Quell-Alpha |
            // |---|---|---|
            // | mit Alphakanal | BLEND | das Pixel-Alpha (`set_alpha` zaehlt hier nicht mit) |
            // | mit Alphakanal | NONE (`set_alpha(None)`) | 255 |
            // | ohne Alphakanal | BLEND (`set_alpha(irgendwas)`) | **0** |
            // | ohne Alphakanal | NONE | 255 |
            //
            // Die dritte Zeile ist kein Tippfehler: ein einziger Aufruf von
            // `set_alpha` kippt das Ergebnis, unabhaengig vom Wert - auch
            // `set_alpha(255)`. pygame liest dort das ungenutzte vierte
            // Byte einer Flaeche ohne Alphakanal, und das ist null.
            // In `alphablit.c` heisst die Bedingung
            // `srcppa = info->src_blend != SDL_BLENDMODE_NONE && srcfmt->Amask`.
            rgba_quell_alpha: if !src.mischen {
                Some(255)
            } else if quelle_hat_alpha {
                None // je Pixel
            } else {
                Some(0)
            },
            colorkey: src.colorkey,
        };

        if gleicher_puffer {
            self.blit_selbst(src, ziel, quelle_start, &info);
        } else {
            let sperre_quell_buf = src.buf.lock();
            let quell_buf = sperre_quell_buf.borrow();
            let sperre_ziel_buf = self.buf.lock();
            let mut ziel_buf = sperre_ziel_buf.borrow_mut();
            blit_roh(
                &mut ziel_buf,
                (self.off.0 as usize, self.off.1 as usize),
                ziel,
                &quell_buf,
                (src.off.0 as usize, src.off.1 as usize),
                quelle_start,
                &info,
            );
        }
        ziel
    }

    /// Ueberlappen Quell- und Zielbereich **im Speicher**?
    ///
    /// Zeilengetreu `surface_do_overlap` aus `src_c/surface.c`, nur in
    /// Pixeln statt in Bytes gerechnet (unser Puffer hat genau eine
    /// Zeilenbreite, `pitch` und `bpp` fallen dabei weg):
    ///
    /// ```c
    /// if (dstpixels <= srcpixels) return 0;
    /// span = w * bpp;
    /// if (dstpixels >= srcpixels + (h - 1) * src->pitch + span) return 0;
    /// dstoffset = (dstpixels - srcpixels) % src->pitch;
    /// return dstoffset < span || dstoffset > src->pitch - span;
    /// ```
    ///
    /// Nur wenn das zutrifft, nimmt pygame fuer einen Blit einer Flaeche
    /// auf sich selbst den **eigenen** Weg statt SDLs - und rechnet dann
    /// anders. Wir selbst blitten ohnehin ueber eine Zwischenkopie, das
    /// Ergebnis ist dasselbe wie bei pygames Richtungsumkehr; hier geht es
    /// allein um die Wahl der Routine.
    fn ueberlappt_mit(&self, src: &Surface, ziel: Rect, quelle_start: (i32, i32)) -> bool {
        let pitch = self.buf.lock().borrow().w as i64;
        let quell_index = (src.off.1 as i64 + quelle_start.1 as i64) * pitch
            + src.off.0 as i64
            + quelle_start.0 as i64;
        let ziel_index =
            (self.off.1 as i64 + ziel.y as i64) * pitch + self.off.0 as i64 + ziel.x as i64;
        if ziel_index <= quell_index {
            return false;
        }
        let span = ziel.w as i64;
        if ziel_index >= quell_index + (ziel.h as i64 - 1) * pitch + span {
            return false;
        }
        let versatz = (ziel_index - quell_index) % pitch;
        versatz < span || versatz > pitch - span
    }

    /// Blit innerhalb desselben Puffers (Flaeche auf sich selbst).
    ///
    /// Die betroffenen Zeilen werden zwischengelagert, damit sich
    /// ueberlappende Bereiche nicht selbst verschmieren.
    fn blit_selbst(&mut self, src: &Surface, ziel: Rect, quelle_start: (i32, i32), info: &BlitInfo) {
        let kopie = {
            let sperre_buf = self.buf.lock();
            let buf = sperre_buf.borrow();
            let mut k = PixelBuf::neu(ziel.w as u32, ziel.h as u32, buf.format);
            k.tafel = buf.tafel.clone();
            for y in 0..ziel.h {
                let zy = src.off.1 as usize + (quelle_start.1 + y) as usize;
                let zx = src.off.0 as usize + quelle_start.0 as usize;
                for x in 0..ziel.w as usize {
                    let v = buf.lies(buf.versatz(zx + x, zy));
                    let off = k.versatz(x, y as usize);
                    k.setz(off, v);
                }
            }
            k
        };
        let sperre_ziel_buf = self.buf.lock();
        let mut ziel_buf = sperre_ziel_buf.borrow_mut();
        blit_roh(
            &mut ziel_buf,
            (self.off.0 as usize, self.off.1 as usize),
            ziel,
            &kopie,
            (0, 0),
            (0, 0),
            info,
        );
    }

    // ----- Kopien und Ausschnitte ----------------------------------------

    /// Eigenstaendige Kopie (teilt keinen Speicher).
    pub fn copy(&self) -> Surface {
        let mut neu = Surface::with_flags(self.size, self.flags);
        // **Erst das Format, dann die Pixel.** Vorher stand es umgekehrt,
        // und das ging gut, solange jeder Puffer ARGB war. Jetzt ist die
        // Bytebreite Teil der Ablage: eine Achtbitflaeche in einen frisch
        // angelegten Vierbytepuffer zu kopieren hiesse, ein Viertel der
        // Zeile zu treffen.
        neu.format = self.format;
        neu.palette = self.palette.clone();
        neu.buf
            .lock()
            .borrow_mut()
            .umstellen(self.format, self.palette.clone());
        {
            let sperre_quelle = self.buf.lock();
            let quelle = sperre_quelle.borrow();
            let sperre_ziel = neu.buf.lock();
            let mut ziel = sperre_ziel.borrow_mut();
            let spanne = self.size.0 as usize * quelle.bpp as usize;
            for y in 0..self.size.1 as usize {
                let von = quelle.versatz(self.off.0 as usize, self.off.1 as usize + y);
                let nach = ziel.versatz(0, y);
                // Byteweise, nicht wortweise: bei drei Bytes je Pixel
                // faellt eine Zeile nicht auf Wortgrenzen.
                let q: &[u8] = bytemuck::cast_slice(&quelle.data);
                let stueck = q[von..von + spanne].to_vec();
                let z: &mut [u8] = bytemuck::cast_slice_mut(&mut ziel.data);
                z[nach..nach + spanne].copy_from_slice(&stueck);
            }
        }
        neu.alpha = self.alpha;
        neu.mischen = self.mischen;
        neu.colorkey = self.colorkey;
        neu.clip = neu.get_rect();
        neu
    }

    /// Kopie ohne Alphakanal.
    ///
    /// In pygame passt `convert()` das Pixelformat ans Fenster an. Hier
    /// gibt es nur ein Format, also ist es eine billige Kopie - die aus
    /// pygame bekannte Falle "du hast convert() vergessen" existiert nicht.
    pub fn convert(&self) -> Surface {
        let mut neu = self.copy();
        neu.flags &= !SRCALPHA;
        // **Das Ergebnis hat immer das Anzeigeformat** - hier also 32 Bit
        // ohne Alphakanal, ganz gleich, womit die Quelle angefangen hat.
        //
        // Gemessen ueber alle vier Tiefen:
        //
        // ```text
        // Surface((4,4), 0, d).convert().get_bitsize()  ->  32   fuer d = 8, 16, 24, 32
        // ```
        //
        // Genau dafuer gibt es `convert`: es bringt eine Flaeche auf das
        // Format, in dem geblittet wird. Eine 8-Bit-Flaeche behielt hier
        // ihre Tiefe **und ihre Farbtafel** und war danach immer noch
        // langsam zu blitten - die Umwandlung hiess so, tat aber nichts.
        //
        // Die Farbtafel faellt mit weg; eine 32-Bit-Flaeche hat keine.
        neu.umformen(Format::XRGB8888, None);
        // Ohne Alphakanal gibt es kein Alpha mehr - das vierte Byte wird
        // geleert, nicht auf 255 gesetzt. Siehe [`Surface::abbilden`]:
        // pygame legt dort eine Null ab, und `scale2x` vergleicht ganze
        // Woerter.
        // Das Leeren des vierten Bytes besorgt `von_argb` in `umstellen`:
        // bei XRGB8888 maskiert es. Die Schleife, die hier stand, war
        // damit doppelt gemoppelt.
        neu
    }

    /// Der **Tafelindex** eines Pixels - fuer das Speichern als
    /// Palettenbild.
    ///
    /// Gibt `0`, wenn die Flaeche keine Tafel hat. Dieselbe Suche wie beim
    /// Einrasten, nur wird die Stelle gebraucht und nicht die Farbe.
    pub fn naechster_tafelindex(&self, pos: (i32, i32)) -> u32 {
        match &self.palette {
            None => 0,
            Some(t) => naechster_index(t, self.get_at_mapped(pos)),
        }
    }

    /// Wechselt die Farbtafel und **nimmt die Pixel mit**.
    ///
    /// pygame speichert bei 8 Bit **Indizes**; wechselt die Tafel, aendert
    /// sich damit die Farbe jedes Pixels, ohne dass jemand Pixel anfasst.
    /// Gemessen:
    ///
    /// ```python
    /// s = Surface((2, 2), depth=8)          # alles Index 0
    /// s.set_palette(256 * [(10, 20, 30)])
    /// s.get_at((1, 0))                      # (10, 20, 30, 255)
    /// ```
    ///
    /// rustgame speichert stattdessen **Farben** (ARGB) und rastet beim
    /// Schreiben auf die Tafel ein - das ist an jeder anderen Stelle die
    /// einfachere Wahl und hier die teurere: ein unberuehrtes Pixel bleibt
    /// schwarz, wo pygame die neue Tafelfarbe zeigt.
    ///
    /// Also wird beim Tafelwechsel umgerechnet: alte Farbe -> Index ->
    /// neue Farbe. Das ist genau die Buchfuehrung, die pygame sich
    /// spart - und sie ist eindeutig, solange die alte Tafel keine
    /// doppelten Farben hat. Hat sie doch welche (pygames Vorgabetafel
    /// fuehrt jede ihrer 128 Farben zweimal), gewinnt der **erste**
    /// Eintrag; sichtbar ist das nicht, denn beide Indizes zeigen
    /// dieselbe Farbe.
    pub fn palette_wechseln(&mut self, neu_tafel: Vec<Color>) {
        let alt_tafel = match &self.palette {
            None => {
                self.palette = Some(neu_tafel);
                return;
            }
            Some(t) => t.clone(),
        };
        // **Die Indizes bleiben stehen, die Tafel wechselt.** Frueher
        // lagen hier Farben, und jede musste ueber `naechster_index` auf
        // ihren Platz in der alten Tafel zurueckgerechnet werden. Seit die
        // Indizes selbst im Speicher stehen, ist das nicht mehr noetig -
        // und auch nicht mehr richtig: ein Index, der auf eine doppelte
        // Farbe zeigte, waere dabei auf den ersten Zwilling gerutscht.
        let _ = &alt_tafel;
        self.palette = Some(neu_tafel.clone());
        self.buf.lock().borrow_mut().tafel = Some(neu_tafel);
    }

    /// Eine Kopie **in einem gegebenen Format**.
    ///
    /// `convert()` ohne Argument nimmt das Anzeigeformat; mit Argument
    /// nimmt pygame genau das genannte. Gemessen an einer Flaeche mit
    /// `(255, 50, 0)`:
    ///
    /// ```text
    /// convert()    -> 32 Bit, (255, 50, 0)
    /// convert(8)   ->  8 Bit, (255, 36, 0)   (auf die Farbtafel gerastet)
    /// convert(16)  -> 16 Bit, (255, 48, 0)   (RGB565)
    /// convert(24)  -> 24 Bit, (255, 50, 0)
    /// ```
    ///
    /// Das Rastern ist der eigentliche Zweck: eine Flaeche soll hinterher
    /// nur noch Farben enthalten, die ihr Format auch darstellen kann.
    pub fn convert_zu(&self, format: Format, palette: Option<Vec<Color>>) -> Surface {
        let mut neu = self.copy();
        neu.flags &= !SRCALPHA;
        if format.masks.3 != 0 {
            neu.flags |= SRCALPHA;
        }
        // Jeden Pixel auf das neue Format bringen - sonst meldet die
        // Flaeche ein Format, das zu ihrem Inhalt nicht passt, und
        // `get_at` rechnet etwas anderes heraus, als drinsteht. Das
        // Einrasten besorgt `umformen`: es schickt jeden Wert durch
        // `von_argb`, und das ist genau die Quantisierung des Formats.
        neu.umformen(format, palette);
        neu
    }

    /// Kopie mit Alphakanal.
    pub fn convert_alpha(&self) -> Surface {
        let mut neu = self.copy();
        let war_ohne_alpha = !self.has_alpha();
        neu.flags |= SRCALPHA;
        // Das Gegenstueck zu `convert`: der Alphakanal kommt (zurueck).
        // Bei einem Format, fuer das pygame keine Alphamasken kennt (8,
        // 12, 15, 24 Bit), wird daraus ARGB8888 - genau das tut pygames
        // `convert_alpha` auch, es waehlt das Anzeigeformat mit Alpha.
        // Und hier ebenso das Anzeigeformat **mit** Alphakanal: gemessen
        // gibt `Surface((4,4), 0, 16).convert_alpha().get_bitsize()` in
        // pygame **32**, nicht 16, und die Masken sind die von ARGB8888.
        //
        // Hier stand `Format::standard(neu.format.bitsize, true)`, also
        // "dieselbe Tiefe, nur mit Alpha" - eine naheliegende Lesart des
        // Namens, die bei 16 Bit ARGB4444 ergab. Der Vermerk daneben sagte
        // sogar schon das Richtige ("es waehlt das Anzeigeformat mit
        // Alpha"), nur stand es nicht im Code.
        neu.umformen(Format::ARGB8888, None);
        // **Der Punkt, an dem das ungenutzte Byte zum echten Alpha wird.**
        // Kam die Quelle ohne Alphakanal, steht dort eine Null - die jetzt
        // als "voellig durchsichtig" gelesen wuerde. pygame liefert an
        // dieser Stelle eine undurchsichtige Flaeche, also muss das Byte
        // beim Formatwechsel gesetzt werden.
        if war_ohne_alpha {
            let sperre_buf = neu.buf.lock();
            let mut buf = sperre_buf.borrow_mut();
            let (w, h) = (buf.w as usize, buf.h as usize);
            for y in 0..h {
                for x in 0..w {
                    let off = buf.versatz(x, y);
                    let v = buf.lies(off) | 0xFF00_0000;
                    buf.setz(off, v);
                }
            }
        }
        // `convert_alpha` liefert in pygame immer eine Flaeche mit
        // `SDL_BLENDMODE_BLEND` - sonst wuerde sie beim Blitten kopiert
        // statt gemischt.
        neu.mischen = true;
        neu
    }

    /// Ein Ausschnitt, der sich den Speicher mit dieser Flaeche **teilt**.
    ///
    /// Schreiben in die Teilflaeche veraendert das Original - genau wie in
    /// pygame.
    pub fn subsurface(&self, rect: Rect) -> Surface {
        let r = rect.normalized().clip(self.get_rect());
        Surface {
            buf: Arc::clone(&self.buf),
            off: (
                self.off.0 + r.x.max(0) as u32,
                self.off.1 + r.y.max(0) as u32,
            ),
            size: (r.w.max(0) as u32, r.h.max(0) as u32),
            flags: self.flags,
            alpha: self.alpha,
            mischen: self.mischen,
            colorkey: self.colorkey,
            clip: Rect::new(0, 0, r.w.max(0), r.h.max(0)),
            parent: Some(Box::new(self.clone())),
            format: self.format,
            palette: self.palette.clone(),
        }
    }

    /// Verschiebt den Inhalt; frei werdende Bereiche bleiben stehen
    /// (wie in pygame).
    pub fn scroll(&mut self, dx: i32, dy: i32) {
        if dx == 0 && dy == 0 {
            return;
        }
        // **Nur innerhalb des Clip-Rechtecks.** Gemessen an echtem pygame,
        // Flaeche 8x4, Zeileninhalt 01234567, Clip (2,1,4,2):
        //
        // ```text
        //   scroll( 2, 0) -> 01232367   (nur Spalten 2..5, nur Zeilen 1..2)
        //   scroll(-2, 0) -> 01454567
        // ```
        //
        // Ohne Clip waere daraus 01012345 geworden. Der differenzielle
        // Fuzzer hat das aufgedeckt - kein Referenzfall kam darauf.
        let c = self.clip;
        if dx.abs() >= c.w || dy.abs() >= c.h {
            return;
        }

        // Und es wird **roh verschoben**, nicht geblittet: pygame benutzt
        // `memmove`. Ueber `blit` zu gehen waere falsch, sobald die Flaeche
        // ein Alpha oder eine Schluesselfarbe traegt - dann wuerde
        // gemischt statt verschoben.
        let breite = (c.w - dx.abs()) as usize;
        let hoehe = c.h - dy.abs();
        let qx = (c.x + (-dx).max(0)) as usize;
        let qy = c.y + (-dy).max(0);
        let zx = (c.x + dx.max(0)) as usize;
        let zy = c.y + dy.max(0);

        self.mit_pixeln(|daten, stride, (ox, oy)| {
            // Die Zeilen so durchgehen, dass sich nichts selbst
            // ueberschreibt: wandert der Inhalt nach unten, zuerst die
            // unterste Zeile. Innerhalb einer Zeile erledigt das
            // `copy_within` (memmove) von selbst.
            for i in 0..hoehe {
                let y = if dy > 0 { hoehe - 1 - i } else { i };
                let q = (oy + (qy + y) as usize) * stride + ox + qx;
                let z = (oy + (zy + y) as usize) * stride + ox + zx;
                daten.copy_within(q..q + breite, z);
            }
        });
    }

    /// `scroll(dx, dy, SCROLL_REPEAT)` - der Inhalt laeuft **rundherum**.
    ///
    /// Alle drei Varianten von `scroll` achten aufs Clip-Rechteck, und
    /// zwar jede auf ihre Weise - gemessen an einer 4x3-Flaeche, deren
    /// Zeile `[1, 11, 21, 31]` lautet, mit `dx = 1`:
    ///
    /// ```text
    /// ohne Clip        Flagge 0  -> [ 1,  1, 11, 21]   (Rand bleibt stehen)
    ///                  REPEAT    -> [31,  1, 11, 21]   (laeuft herum)
    ///                  ERASE     -> [ 0,  1, 11, 21]   (Rand wird Null)
    /// Clip (1,0,2,3)   Flagge 0  -> [ 1, 11, 11, 31]
    ///                  REPEAT    -> [ 1, 21, 11, 31]   (herum **im Clip**)
    ///                  ERASE     -> [ 1,  0, 11, 31]
    /// ```
    ///
    /// Das Herumlaufen gilt also nur innerhalb des Clips, nicht ueber die
    /// ganze Flaeche - der Teil ausserhalb bleibt unberuehrt.
    ///
    /// Und der Versatz wird **modulo** genommen: `dx = 5` auf einer
    /// vier Pixel breiten Flaeche verhaelt sich wie `dx = 1`. Genau darin
    /// unterscheidet sich diese Variante von den anderen beiden, die bei
    /// `|dx| >= Breite` gar nichts mehr tun (Flagge 0) beziehungsweise
    /// alles loeschen (`ERASE`).
    pub fn scroll_wiederholend(&mut self, dx: i32, dy: i32) {
        let c = self.clip;
        if c.w <= 0 || c.h <= 0 {
            return;
        }
        // `rem_euclid`, nicht `%`: bei `-1 % 4` gibt Rust `-1`, gebraucht
        // wird `3`. Mit `%` liefe die Maske nach der falschen Seite.
        let vx = dx.rem_euclid(c.w);
        let vy = dy.rem_euclid(c.h);
        if vx == 0 && vy == 0 {
            return;
        }
        // Erst lesen, dann schreiben: beim Herumlaufen ueberschreibt sich
        // jede Reihenfolge selbst, ein `copy_within` genuegt hier also
        // nicht. Der Zwischenspeicher ist so gross wie der Clip, nicht wie
        // die Flaeche.
        let (bw, bh) = (c.w as usize, c.h as usize);
        let (cx, cy) = (c.x as usize, c.y as usize);
        self.mit_pixeln(|daten, stride, (ox, oy)| {
            let mut vorher = vec![0u32; bw * bh];
            for y in 0..bh {
                let q = (oy + cy + y) * stride + ox + cx;
                vorher[y * bw..(y + 1) * bw].copy_from_slice(&daten[q..q + bw]);
            }
            for y in 0..bh {
                let qy = (y + bh - vy as usize) % bh;
                let z = (oy + cy + y) * stride + ox + cx;
                for x in 0..bw {
                    let qx = (x + bw - vx as usize) % bw;
                    daten[z + x] = vorher[qy * bw + qx];
                }
            }
        });
    }

    /// `scroll(dx, dy, SCROLL_ERASE)` - der freigewordene Rand wird **Null**.
    ///
    /// Null heisst hier der rohe Pixelwert 0, also durchsichtiges Schwarz
    /// auf einer Flaeche mit Alphakanal und Schwarz ohne. Gemessen gibt
    /// pygame dort `(0, 0, 0, 0)` beziehungsweise `(0, 0, 0, 255)` - es
    /// faerbt also nicht mit einer Farbe, sondern loescht Speicher.
    ///
    /// Anders als bei Flagge 0 ist `|dx| >= Breite` hier **kein**
    /// Nichtstun: dann ist der ganze Clip freigeworden und wird geloescht.
    /// Die Vergleichstabelle steht bei [`Surface::scroll_wiederholend`].
    pub fn scroll_loeschend(&mut self, dx: i32, dy: i32) {
        let c = self.clip;
        if c.w <= 0 || c.h <= 0 || (dx == 0 && dy == 0) {
            return;
        }
        let (bw, bh) = (c.w as usize, c.h as usize);
        let (cx, cy) = (c.x as usize, c.y as usize);
        self.mit_pixeln(|daten, stride, (ox, oy)| {
            // Derselbe Weg wie beim Herumlaufen - erst den alten Stand
            // festhalten - nur dass hier alles, was von aussen kaeme,
            // Null wird. Das ist einfacher als Verschieben und dann
            // Nachloeschen, und es stimmt auch, wenn sich Quelle und Ziel
            // ueberlappen.
            let mut vorher = vec![0u32; bw * bh];
            for y in 0..bh {
                let q = (oy + cy + y) * stride + ox + cx;
                vorher[y * bw..(y + 1) * bw].copy_from_slice(&daten[q..q + bw]);
            }
            for y in 0..bh {
                let z = (oy + cy + y) * stride + ox + cx;
                let qy = y as i32 - dy;
                for x in 0..bw {
                    let qx = x as i32 - dx;
                    let drin = qx >= 0 && qy >= 0 && qx < c.w && qy < c.h;
                    daten[z + x] = if drin {
                        vorher[qy as usize * bw + qx as usize]
                    } else {
                        0
                    };
                }
            }
        });
    }

    /// Das kleinste Rechteck, das alle Pixel mit Alpha ueber `min_alpha`
    /// umschliesst.
    /// Das kleinste Rechteck um alles, was **sichtbar** ist.
    ///
    /// Sichtbar heisst zweierlei, und das zweite fehlte hier: Alpha ab
    /// `min_alpha` **und** nicht die Schluesselfarbe. Gemessen an einer
    /// 24-Bit-Flaeche, ganz schwarz gefuellt:
    ///
    /// ```text
    /// ohne Schluesselfarbe        -> Rect(0, 0, 70, 70)
    /// nach set_colorkey((0,0,0))  -> Rect(0, 0, 0, 0)
    /// danach set_at((30,30), weiss) -> Rect(30, 30, 1, 1)
    /// ```
    ///
    /// Eine Flaeche ohne Alphakanal hat ueberall Alpha 255, also war die
    /// Antwort vorher immer die ganze Flaeche - eine Auskunft, die stimmt,
    /// solange niemand eine Schluesselfarbe setzt.
    pub fn get_bounding_rect(&self, min_alpha: u8) -> Rect {
        let sperre_buf = self.buf.lock();
        let buf = sperre_buf.borrow();
        let (mut x1, mut y1) = (self.size.0 as i32, self.size.1 as i32);
        let (mut x2, mut y2) = (-1i32, -1i32);
        for y in 0..self.size.1 as i32 {
            for x in 0..self.size.0 as i32 {
                let v = buf.nach_argb(buf.lies(buf.versatz(
                    self.off.0 as usize + x as usize,
                    self.off.1 as usize + y as usize,
                )));
                // **Ueber `sichtbar`, nicht roh.** Ohne Alphakanal steht im
                // vierten Byte eine Null; wer es roh liest, findet nie ein
                // Pixel und meldet ein leeres Rechteck. pygame liefert
                // dort **immer die ganze Flaeche** - gemessen, sogar bei
                // `min_alpha = 255`.
                //
                // Und der Vergleich ist `>=`, nicht `>`: gemessen mit den
                // Alphawerten 5 und 200 ergibt `min_alpha = 5` noch
                // `(2,1,3,3)`, erst 6 schneidet den ersten Pixel weg.
                // Und die **Schluesselfarbe** zaehlt als unsichtbar - siehe
                // den Vermerk oben. Verglichen werden nur die drei
                // Farbkanaele, wie beim Blit.
                if let Some(k) = self.colorkey {
                    if v & 0x00FF_FFFF == k & 0x00FF_FFFF {
                        continue;
                    }
                }
                if (self.sichtbar(v) >> 24) as u8 >= min_alpha {
                    x1 = x1.min(x);
                    y1 = y1.min(y);
                    x2 = x2.max(x);
                    y2 = y2.max(y);
                }
            }
        }
        if x2 < 0 {
            Rect::new(0, 0, 0, 0)
        } else {
            Rect::new(x1, y1, x2 - x1 + 1, y2 - y1 + 1)
        }
    }

    /// Die Pixel als RGBA-Bytes (fuer `image::tobytes` und Tests).
    pub fn to_rgba_bytes(&self) -> Vec<u8> {
        let sperre_buf = self.buf.lock();
        let buf = sperre_buf.borrow();
        let mut out = Vec::with_capacity((self.size.0 * self.size.1 * 4) as usize);
        // Bei den Standardformaten ist `nach_argb` die Identitaet: die
        // Zeile wird als Wortscheibe gelesen, `sichtbar` einmal
        // entschieden. Dieselben Bytes wie die Schleife darunter.
        if ist_standard32(&buf) {
            let oder = if self.has_alpha() { 0 } else { 0xFF00_0000u32 };
            let zw = (buf.pitch / 4) as usize;
            let (ox, oy, w) = (self.off.0 as usize, self.off.1 as usize, self.size.0 as usize);
            for y in 0..self.size.1 as usize {
                let a = (oy + y) * zw + ox;
                for &v in &buf.data[a..a + w] {
                    let v = v | oder;
                    out.extend_from_slice(&[(v >> 16) as u8, (v >> 8) as u8, v as u8, (v >> 24) as u8]);
                }
            }
            return out;
        }
        for y in 0..self.size.1 as usize {
            for x in 0..self.size.0 as usize {
                let v = buf.nach_argb(
                    buf.lies(buf.versatz(self.off.0 as usize + x, self.off.1 as usize + y)),
                );
                // Maskieren, sonst traegt eine Flaeche ohne Alphakanal ihr
                // ungenutztes viertes Byte nach aussen - pygames `tobytes`
                // tut das nicht.
                let (a, r, g, b) = blend::unpack(self.sichtbar(v));
                out.extend_from_slice(&[r, g, b, a]);
            }
        }
        out
    }

    /// Die Pixel als RGB-Bytes (ohne Alphakanal).
    pub fn to_rgb_bytes(&self) -> Vec<u8> {
        let sperre_buf = self.buf.lock();
        let buf = sperre_buf.borrow();
        let mut out = Vec::with_capacity((self.size.0 * self.size.1 * 3) as usize);
        // Wie in `to_rgba_bytes`: bei den Standardformaten zeilenweise.
        if ist_standard32(&buf) {
            let zw = (buf.pitch / 4) as usize;
            let (ox, oy, w) = (self.off.0 as usize, self.off.1 as usize, self.size.0 as usize);
            for y in 0..self.size.1 as usize {
                let a = (oy + y) * zw + ox;
                for &v in &buf.data[a..a + w] {
                    out.extend_from_slice(&[(v >> 16) as u8, (v >> 8) as u8, v as u8]);
                }
            }
            return out;
        }
        for y in 0..self.size.1 as usize {
            for x in 0..self.size.0 as usize {
                let v = buf.nach_argb(
                    buf.lies(buf.versatz(self.off.0 as usize + x, self.off.1 as usize + y)),
                );
                let (_, r, g, b) = blend::unpack(v);
                out.extend_from_slice(&[r, g, b]);
            }
        }
        out
    }
}

// ---------------------------------------------------------------------------
//  Die Auswahlkette
// ---------------------------------------------------------------------------

/// Welche Blit-Routine dieser Aufruf nimmt.
///
/// pygame hat fuer `surface.blit()` **nicht eine** Routine, sondern sieben,
/// und sie rechnen unterschiedlich. Die Auswahl steht an zwei Stellen im
/// C-Quelltext:
///
/// 1. `src_c/surface.c:pgSurface_Blit` entscheidet, ob pygame den Blit
///    selbst macht (`pygame_Blit` in `alphablit.c`) oder an SDL abgibt
///    (`SDL_BlitSurface`).
/// 2. Innerhalb von `pygame_Blit` waehlt `SoftBlitPyGame` per `switch` aus,
///    und innerhalb von `SDL_BlitSurface` waehlt `SDL_CalculateBlit`.
///
/// Wer nur eine Routine nachbaut, trifft 99 von 100 Pixeln - und liegt beim
/// hundertsten um eins daneben. Die Tabelle steht bei `Blitweg::waehlen`.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Blitweg {
    /// SDL kopiert nur (`SDL_BlitCopy`, `BlitNtoN`, `Blit4to4MaskAlpha`).
    Kopie,
    /// SDL `BlitRGBtoRGBSurfaceAlpha` - Flaechen-Alpha, Ziel-Alpha wird 255.
    SdlFlaechenAlpha,
    /// SDL `BlitNtoNSurfaceAlphaKey` - Flaechen-Alpha mit Schluesselfarbe.
    SdlFlaechenAlphaKey,
    /// SDL `SDL_Blit_Slow` - der Rueckfallweg fuer Kombinationen, die
    /// `SDL_CalculateBlitA` nicht kennt (Alphakanal **und** Schluesselfarbe).
    SdlLangsam,
    /// pygame `alphablit_alpha` - der Weg fuer Quellen mit Alphakanal.
    PygameAlpha,
    /// pygame `alphablit_colorkey`.
    PygameColorkey,
    /// pygame `alphablit_solid`.
    PygameSolid,
    /// Eine `BLEND_*`-Mischart (`SoftBlitPyGame`, `case PYGAME_BLEND_*`).
    Misch(u32),
    /// `BLEND_PREMULTIPLIED` mit einer Quelle ohne Mischmodus: dann
    /// kopiert `blit_blend_premultiplied` nur die drei Farbkanaele und
    /// laesst den Alphakanal des Ziels stehen.
    PremultipliedRgbKopie,
}

impl Blitweg {
    /// Die Auswahlkette, zeilengetreu aus `pgSurface_Blit` und
    /// `SoftBlitPyGame`.
    ///
    /// ```c
    /// /* surface.c, pgSurface_Blit - gekuerzt auf das, was hier zutrifft */
    /// if ((blend_flags != 0 && blend_flags != PYGAME_BLEND_ALPHA_SDL2) ||
    ///     ((SDL_HasColorKey(src) || _PgSurface_SrcAlpha(src) == 1) &&
    ///      dst->pixels == src->pixels && srcrect != NULL &&
    ///      surface_do_overlap(src, srcrect, dst, dstrect, &dstclip)))
    ///     result = pygame_Blit(src, srcrect, dst, dstrect, blend_flags);
    /// else if (... dst 8 Bit ...) { ... }
    /// else if (blend_flags != PYGAME_BLEND_ALPHA_SDL2 && !SDL_HasColorKey(src) &&
    ///          (PG_SURF_BytesPerPixel(dst) == 4 || 2) && _PgSurface_SrcAlpha(src) &&
    ///          SDL_ISPIXELFORMAT_ALPHA(PG_SURF_FORMATENUM(src)) && ...)
    ///     result = pygame_Blit(src, srcrect, dst, dstrect, blend_flags);
    /// else
    ///     result = PG_BlitSurface(src, srcrect, dst, dstrect);
    /// ```
    ///
    /// Daraus wird fuer unsere Welt (alles 32 Bit ARGB, gleiche RGB-Masken)
    /// diese Matrix - jede Zeile an echtem pygame-ce 2.5.7 nachgemessen:
    ///
    /// ```text
    /// Quelle           Schluessel  Mischmodus  Flaechen-Alpha  Routine
    /// ohne Alphakanal  nein        NONE        -               Kopie
    /// ohne Alphakanal  nein        BLEND       255             Kopie
    /// ohne Alphakanal  nein        BLEND       < 255           SdlFlaechenAlpha
    /// ohne Alphakanal  ja          NONE/255    -               Kopie (Schluesselpixel aus)
    /// ohne Alphakanal  ja          BLEND       < 255           SdlFlaechenAlphaKey
    /// mit Alphakanal   nein        NONE        -               Kopie (samt Alpha)
    /// mit Alphakanal   nein        BLEND       egal            PygameAlpha
    /// mit Alphakanal   ja          NONE        -               Kopie (samt Alpha)
    /// mit Alphakanal   ja          BLEND       egal            SdlLangsam
    /// ```
    ///
    /// Die drei `Pygame*`-Wege fuer den Selbstblit stehen in
    /// `SoftBlitPyGame`, `case 0:`.
    ///
    /// `hat_quellrechteck` ist das `srcrect != NULL` aus der Bedingung oben,
    /// und das hiess hier eine ganze Weile `hat_ausschnitt` - also "hat der
    /// Aufrufer ein `area` uebergeben". **Das ist die falsche Frage.**
    /// `surf_blit` und `surf_blits` setzen das Rechteck selbst, wenn keines
    /// kommt (`surface.c:2248`: `temp.w = src->w; src_rect = &temp;`), also
    /// ist es von Python aus immer belegt. `NULL` uebergibt genau **ein**
    /// Aufrufer: `surf_fblits` (`surface.c:2572`). Nachgemessen an einem
    /// ueberlappenden Selbstblit mit Flaechenalpha 128, gruen auf rot:
    ///
    /// ```text
    /// blit                (127, 128, 0)      <- pygames eigener Weg
    /// blits               (127, 128, 0)
    /// blits mit area=None (127, 128, 0)
    /// blit mit area       (127, 128, 0)
    /// fblits              (127, 127, 0)      <- SDLs Weg
    /// ```
    ///
    /// Der Unterschied ist genau ein Summand: SDL rechnet
    /// `((s - d) * a >> 8) + d`, pygames `ALPHA_BLEND_COMP` dagegen
    /// `(((s - d) * a + s) >> 8) + d` (`surface.h:294`). Ein Wert, ein
    /// Pixel - und daran hing `surface_test.test_blanket_alpha` in allen
    /// vier Tiefen.
    fn waehlen(
        flags: u32,
        src: &Surface,
        ziel: &Surface,
        hat_quellrechteck: bool,
        ueberlappt: bool,
    ) -> Blitweg {
        if flags != 0 {
            // "Convert alpha multiply blends to regular blends if either of
            // the surfaces don't have alpha channels" - `SoftBlitPyGame`
            // biegt das direkt vor dem `switch` um. Sichtbar wird es nur am
            // Alphakanal des Ziels: BLEND_RGBA_MULT wuerde ihn mitrechnen,
            // BLEND_MULT laesst ihn stehen.
            if flags == flags::BLEND_RGBA_MULT && (!src.mischen || !ziel.mischen) {
                return Blitweg::Misch(flags::BLEND_RGB_MULT);
            }
            // `blit_blend_premultiplied` faengt mit einem Schnellweg an:
            //
            // ```c
            // if (srcbpp >= 3 && dstbpp >= 3 && info->src_blend == SDL_BLENDMODE_NONE) {
            //     dst[dstoffsetR] = src[srcoffsetR];   /* nur RGB, kein Alpha */
            // ```
            if flags == flags::BLEND_PREMULTIPLIED && !src.mischen {
                return Blitweg::PremultipliedRgbKopie;
            }
            return Blitweg::Misch(flags);
        }

        // Der eigene Weg von pygame - entweder weil die Flaeche sich selbst
        // ueberlappend beblittet, oder weil die Quelle einen Alphakanal hat
        // und keine Schluesselfarbe.
        let selbstblit =
            ueberlappt && hat_quellrechteck && (src.colorkey.is_some() || src.mischen);
        if selbstblit || (src.colorkey.is_none() && src.has_alpha() && src.mischen) {
            // `SoftBlitPyGame`, `case 0:`
            //     if (src_blend != NONE && src->Amask)  -> alphablit_alpha
            //     else if (src_has_colorkey)            -> alphablit_colorkey
            //     else                                  -> alphablit_solid
            return if src.mischen && src.has_alpha() {
                Blitweg::PygameAlpha
            } else if src.colorkey.is_some() {
                Blitweg::PygameColorkey
            } else {
                Blitweg::PygameSolid
            };
        }

        // SDLs Weg. `SDL_COPY_BLEND` ist nur dann wirklich aktiv, wenn die
        // Quelle einen Alphakanal hat oder ein Flaechen-Alpha unter 255
        // gesetzt ist; sonst kopiert SDL. (Gemessen: `set_alpha(255)` auf
        // einer Flaeche ohne Alphakanal kopiert exakt, waehrend
        // `BlitRGBtoRGBSurfaceAlpha` bei 255 um eins danebenlaege.)
        let sdl_mischt = src.mischen && (src.has_alpha() || src.alpha.unwrap_or(255) < 255);
        if !sdl_mischt {
            Blitweg::Kopie
        } else if src.has_alpha() {
            // Alphakanal **und** Schluesselfarbe: diese Kombination kennt
            // `SDL_CalculateBlitA` nicht, also faellt SDL auf den langsamen
            // Blitter zurueck. (Ohne Schluesselfarbe waeren wir oben bei
            // `PygameAlpha` gelandet.)
            Blitweg::SdlLangsam
        } else if src.colorkey.is_some() {
            Blitweg::SdlFlaechenAlphaKey
        } else {
            Blitweg::SdlFlaechenAlpha
        }
    }
}

/// Alles, was der Pixel-Durchlauf wissen muss - Gegenstueck zu
/// `SDL_BlitInfo` aus `src_c/alphablit.c`.
struct BlitInfo {
    weg: Blitweg,
    /// `info.src_blanket_alpha` - das Flaechen-Alpha der Quelle.
    flaechen_alpha: u8,
    quelle_hat_alpha: bool,
    ziel_hat_alpha: bool,
    /// Was die `BLEND_RGBA_*`-Mischarten als Quell-Alpha sehen; `None`
    /// heisst "das Pixel-Alpha".
    rgba_quell_alpha: Option<u8>,
    /// `info.src_colorkey`, bereits als ARGB gepackt.
    colorkey: Option<u32>,
    /// Wie das Ziel seine Farben speichern kann - `None` bei 24 und 32
    /// Bit ohne Farbtafel, also im Normalfall.
    ///
    /// Siehe [`Surface::raster`]. Ohne das schreibt jede Mischart Werte
    /// ins Ziel, die dessen Format gar nicht darstellen kann; sichtbar
    /// wird es beim Zurücklesen, denn `get_at` rechnet dann etwas
    /// anderes heraus, als hineingeschrieben wurde.
    raster: Option<Raster>,
}

/// Der eigentliche Pixel-Durchlauf.
///
/// Absichtlich eine freie Funktion: so ist genau **ein** Ausleih-Vermerk je
/// Puffer offen, und der Blit einer Flaeche auf sich selbst laesst sich
/// ueber eine Zwischenkopie bedienen.
/// Vier Bytes je Pixel, keine Farbtafel, eines der zwei Standardformate.
///
/// Fuer genau diese Puffer sind [`PixelBuf::lies`] und
/// [`PixelBuf::nach_argb`] die Identitaet und [`PixelBuf::von_argb`]
/// hoechstens eine Maske - und das ist das Format jeder
/// `Surface((w, h))` und jeder `SRCALPHA`-Flaeche.
fn ist_standard32(b: &PixelBuf) -> bool {
    b.bpp == 4
        && b.tafel.is_none()
        && (b.format == Format::ARGB8888 || b.format == Format::XRGB8888)
}

/// Was `blit_roh` vor der Schleife ausrechnet und `blit_32` mitbekommt.
#[derive(Clone, Copy)]
struct Vorlauf {
    roh_alpha: bool,
    liest_roh: bool,
    ueberspringen: Option<u32>,
}

/// Der 32-Bit-Weg von [`blit_roh`].
///
/// ## Warum es ihn gibt
///
/// Gemessen in Runde 18 an PyGameZ selbst (2048, 1280x960, UI v4.2):
/// pygame **1,5 ms** je Bild, rustgame **38,8 ms**, und davon **96 %** in
/// `Surface.blit` - 10.000 Blits kosteten 7,9 s gegen 0,25 s. Die
/// allgemeine Schleife fragt **je Pixel** das Format beider Puffer ab
/// (`nach_argb`, `von_argb` vergleichen dazu ganze `Format`-Werte), die
/// Farbtafel, den Blitweg und die Mischart - fuer jeden der 1,2 Millionen
/// Pixel eines Vollbild-Hintergrunds aufs Neue.
///
/// ## Was er tut
///
/// **Dieselbe Rechnung**, Zeile fuer Zeile aus der Schleife in
/// `blit_roh` abgeschrieben - nur faellt jede Weiche, die fuer die ganze
/// Flaeche gleich ausgeht, **vor** der Schleife: der Weg und die Mischart
/// waehlen eine Rechnung, die als Nulltyp in die Schleife eingesetzt wird
/// ([`blend::BlendOp`]), und Lesen und Schreiben sind fuer die zwei
/// Standardformate Identitaet bzw. eine Maske. Kein einziges Ergebnis
/// darf sich dadurch aendern; geprueft wird das von denselben
/// Messgeraeten wie vorher (Referenzbilder, Fuzzer, Modulvergleich,
/// PyGameZ Bild fuer Bild).
#[allow(clippy::too_many_arguments)]
fn blit_32(
    ziel_buf: &mut PixelBuf,
    ziel_off: (usize, usize),
    ziel: Rect,
    quell_buf: &PixelBuf,
    quell_off: (usize, usize),
    quell_start: (i32, i32),
    info: &BlitInfo,
    v: Vorlauf,
) {
    let wo = Lage32 {
        zw: (ziel_buf.pitch / 4) as usize,
        qw: (quell_buf.pitch / 4) as usize,
        ziel_off,
        ziel,
        quell_off,
        quell_start,
    };
    // Bei XRGB8888 laesst `von_argb` das vierte Byte fallen.
    let ziel_maske = if ziel_buf.format == Format::ARGB8888 {
        0xFFFF_FFFFu32
    } else {
        0x00FF_FFFF
    };
    let k = Kern32 {
        flaechen_alpha: info.flaechen_alpha,
        quelle_hat_alpha: info.quelle_hat_alpha,
        ziel_alpha_setzen: v.roh_alpha && !v.liest_roh,
        ueberspringen: v.ueberspringen,
        // Der Nachlauf von `blit_roh`, in zwei Masken gefasst: was mit dem
        // vierten Byte eines Ziels ohne Alphakanal geschieht, haengt am
        // Weg, und `von_argb` laesst es bei XRGB8888 ganz fallen.
        nach_oder: if v.roh_alpha && info.weg == Blitweg::SdlFlaechenAlpha {
            0xFF00_0000
        } else {
            0
        },
        nach_und: if v.roh_alpha && info.weg != Blitweg::SdlFlaechenAlpha && !v.liest_roh {
            0x00FF_FFFF & ziel_maske
        } else {
            ziel_maske
        },
    };

    let fa = info.flaechen_alpha;
    // Was die `BLEND_RGBA_*`-Mischarten im Alphakanal der Quelle sehen -
    // dieselbe Tabelle wie in `blit_roh`.
    let rgba = info.rgba_quell_alpha;
    let rgba_quelle = move |s: u32| match rgba {
        None => s,
        Some(a) => (s & 0x00FF_FFFF) | ((a as u32) << 24),
    };
    let d = ziel_buf;
    let q = quell_buf;
    match info.weg {
        Blitweg::Kopie => {
            let qa = info.quelle_hat_alpha;
            k.treibe(d, q, &wo, move |_d, s, _w| {
                if qa {
                    s
                } else {
                    (s & 0x00FF_FFFF) | ((fa as u32) << 24)
                }
            })
        }
        Blitweg::PremultipliedRgbKopie => {
            k.treibe(d, q, &wo, |d, s, _w| (d & 0xFF00_0000) | (s & 0x00FF_FFFF))
        }
        Blitweg::SdlFlaechenAlpha => k.treibe(d, q, &wo, blend::SdlFlaechenAlpha::apply),
        Blitweg::SdlFlaechenAlphaKey => k.treibe(d, q, &wo, blend::SdlFlaechenAlphaKey::apply),
        Blitweg::SdlLangsam => k.treibe(d, q, &wo, blend::SdlLangsam::apply),
        Blitweg::PygameAlpha | Blitweg::PygameSolid => {
            k.treibe(d, q, &wo, blend::PygameAlphaBlend::apply)
        }
        Blitweg::PygameColorkey => {
            let key = info.colorkey;
            k.treibe(d, q, &wo, move |d, s, _w| {
                let a = match key {
                    Some(k) if s == k => 0,
                    _ => fa,
                };
                blend::PygameAlphaBlend::apply(d, s, a)
            })
        }
        Blitweg::Misch(f) => match f {
            flags::BLEND_RGB_ADD => k.treibe(d, q, &wo, blend::RgbAdd::apply),
            flags::BLEND_RGB_SUB => k.treibe(d, q, &wo, blend::RgbSub::apply),
            flags::BLEND_RGB_MULT => k.treibe(d, q, &wo, blend::RgbMult::apply),
            flags::BLEND_RGB_MIN => k.treibe(d, q, &wo, blend::RgbMin::apply),
            flags::BLEND_RGB_MAX => k.treibe(d, q, &wo, blend::RgbMax::apply),
            flags::BLEND_RGBA_ADD => k.treibe(d, q, &wo, move |d, s, w| {
                blend::RgbaAdd::apply(d, rgba_quelle(s), w)
            }),
            flags::BLEND_RGBA_SUB => k.treibe(d, q, &wo, move |d, s, w| {
                blend::RgbaSub::apply(d, rgba_quelle(s), w)
            }),
            flags::BLEND_RGBA_MULT => k.treibe(d, q, &wo, move |d, s, w| {
                blend::RgbaMult::apply(d, rgba_quelle(s), w)
            }),
            flags::BLEND_RGBA_MIN => k.treibe(d, q, &wo, move |d, s, w| {
                blend::RgbaMin::apply(d, rgba_quelle(s), w)
            }),
            flags::BLEND_RGBA_MAX => k.treibe(d, q, &wo, move |d, s, w| {
                blend::RgbaMax::apply(d, rgba_quelle(s), w)
            }),
            flags::BLEND_PREMULTIPLIED => k.treibe(d, q, &wo, move |d, s, w| {
                blend::Premultiplied::apply(d, rgba_quelle(s), w)
            }),
            flags::BLEND_ALPHA_SDL2 => k.treibe(d, q, &wo, blend::SdlPixelAlpha::apply),
            _ => k.treibe(d, q, &wo, blend::PygameAlphaBlend::apply),
        },
    }
}

/// Wo in den zwei Puffern gelesen und geschrieben wird - in Worten.
struct Lage32 {
    zw: usize,
    qw: usize,
    ziel_off: (usize, usize),
    ziel: Rect,
    quell_off: (usize, usize),
    quell_start: (i32, i32),
}

/// Die Weichen von `blit_roh`, fuer die ganze Flaeche einmal entschieden.
struct Kern32 {
    flaechen_alpha: u8,
    quelle_hat_alpha: bool,
    /// `roh_alpha && !liest_roh`: das Ziel wird als undurchsichtig gelesen.
    ziel_alpha_setzen: bool,
    ueberspringen: Option<u32>,
    nach_oder: u32,
    nach_und: u32,
}

impl Kern32 {
    /// Die Pixelschleife - Zeile fuer Zeile dieselbe wie in `blit_roh`.
    ///
    /// `rechnung` bekommt das gelesene Ziel, die Quelle und das wirksame
    /// Quell-Alpha und gibt das ARGB-Wort zurueck, wie es der `match`
    /// dort ausrechnet. Weil `F` ein eigener Typ je Aufrufstelle ist, setzt
    /// der Uebersetzer die Rechnung in die Schleife ein.
    #[inline(always)]
    fn treibe<F: Fn(u32, u32, u8) -> u32>(
        &self,
        ziel_buf: &mut PixelBuf,
        quell_buf: &PixelBuf,
        wo: &Lage32,
        rechnung: F,
    ) {
        // Der haeufigste Fall - keine Schluesselfarbe, kein Flaechen-Alpha -
        // bekommt je Kombination der zwei verbliebenen Schalter eine
        // **weichenfreie** Schleife, die der Uebersetzer vektorisieren
        // kann. Alles andere laeuft durch die allgemeine Schleife darunter.
        if self.ueberspringen.is_none() && self.flaechen_alpha == 255 {
            match (self.ziel_alpha_setzen, self.quelle_hat_alpha) {
                (false, false) => self.glatt::<false, false, F>(ziel_buf, quell_buf, wo, rechnung),
                (false, true) => self.glatt::<false, true, F>(ziel_buf, quell_buf, wo, rechnung),
                (true, false) => self.glatt::<true, false, F>(ziel_buf, quell_buf, wo, rechnung),
                (true, true) => self.glatt::<true, true, F>(ziel_buf, quell_buf, wo, rechnung),
            }
            return;
        }
        self.allgemein(ziel_buf, quell_buf, wo, rechnung);
    }

    /// Die Schleife ohne Schluesselfarbe und mit Flaechen-Alpha 255.
    ///
    /// Dann ist das wirksame Alpha das Pixel-Alpha (oder 255 ohne
    /// Alphakanal) - genau was die allgemeine Schleife in diesem Fall
    /// auch ausrechnet, nur ohne eine einzige Weiche je Pixel.
    #[inline(always)]
    fn glatt<const SETZEN: bool, const QA: bool, F: Fn(u32, u32, u8) -> u32>(
        &self,
        ziel_buf: &mut PixelBuf,
        quell_buf: &PixelBuf,
        wo: &Lage32,
        rechnung: F,
    ) {
        let breite = wo.ziel.w as usize;
        let (oder, und) = (self.nach_oder, self.nach_und);
        for y in 0..wo.ziel.h as usize {
            let zi = (wo.ziel_off.1 + wo.ziel.y as usize + y) * wo.zw
                + wo.ziel_off.0
                + wo.ziel.x as usize;
            let qi = (wo.quell_off.1 + wo.quell_start.1 as usize + y) * wo.qw
                + wo.quell_off.0
                + wo.quell_start.0 as usize;
            let q = &quell_buf.data[qi..qi + breite];
            let z = &mut ziel_buf.data[zi..zi + breite];
            for (dz, &s) in z.iter_mut().zip(q) {
                let d = if SETZEN { *dz | 0xFF00_0000 } else { *dz };
                let wirksam = if QA { (s >> 24) as u8 } else { 255 };
                *dz = (rechnung(d, s, wirksam) | oder) & und;
            }
        }
    }

    /// Die allgemeine Schleife - jeder Fall, Zeile fuer Zeile wie in
    /// `blit_roh`.
    #[inline(always)]
    fn allgemein<F: Fn(u32, u32, u8) -> u32>(
        &self,
        ziel_buf: &mut PixelBuf,
        quell_buf: &PixelBuf,
        wo: &Lage32,
        rechnung: F,
    ) {
        let breite = wo.ziel.w as usize;
        for y in 0..wo.ziel.h as usize {
            let zi = (wo.ziel_off.1 + wo.ziel.y as usize + y) * wo.zw
                + wo.ziel_off.0
                + wo.ziel.x as usize;
            let qi = (wo.quell_off.1 + wo.quell_start.1 as usize + y) * wo.qw
                + wo.quell_off.0
                + wo.quell_start.0 as usize;
            let q = &quell_buf.data[qi..qi + breite];
            let z = &mut ziel_buf.data[zi..zi + breite];
            for (dz, &s) in z.iter_mut().zip(q) {
                if let Some(key) = self.ueberspringen {
                    // SDL vergleicht ohne den Alphakanal.
                    if s & 0x00FF_FFFF == key & 0x00FF_FFFF {
                        continue;
                    }
                }
                let d = if self.ziel_alpha_setzen {
                    *dz | 0xFF00_0000
                } else {
                    *dz
                };
                let pixel_alpha = if self.quelle_hat_alpha {
                    (s >> 24) as u8
                } else {
                    255
                };
                let wirksam = if self.flaechen_alpha == 255 {
                    pixel_alpha
                } else if pixel_alpha == 255 {
                    self.flaechen_alpha
                } else {
                    ((pixel_alpha as u32 * self.flaechen_alpha as u32) / 255) as u8
                };
                let neu = rechnung(d, s, wirksam);
                *dz = (neu | self.nach_oder) & self.nach_und;
            }
        }
    }
}

fn blit_roh(
    ziel_buf: &mut PixelBuf,
    ziel_off: (usize, usize),
    ziel: Rect,
    quell_buf: &PixelBuf,
    quell_off: (usize, usize),
    quell_start: (i32, i32),
    info: &BlitInfo,
) {
    // **Byteversaetze, keine Pixelnummern.** Seit die Puffer Bytes im
    // gemeldeten Format halten, ist eine Zeile `pitch` Bytes lang und ein
    // Pixel `bpp` Bytes breit. Bei 32 Bit sind `pitch == 4*w` und
    // `bpp == 4`, die Rechnung ist also dieselbe wie vorher mal vier.
    let zb = ziel_buf.pitch as usize;
    let zbpp = ziel_buf.bpp as usize;
    // Ohne eigenen Alphakanal bleibt das Ziel undurchsichtig - der Wert
    // mischt zwar mit, wird aber nicht abgelegt. Gemessen an echtem pygame.
    // Ohne Alphakanal am Ziel legen die **zuordnenden** Schreibwege
    // (normaler Blit, alles ueber `PG_MapRGBA`) eine Null ins vierte Byte;
    // die `BLEND_RGBA_*`-Mischarten dagegen rechnen roh darauf weiter und
    // lassen stehen, was herauskommt. Gemessen am Rohspeicher, Ziel
    // `(72,197,142)` ohne Alphakanal, Quelle mit Alpha 148:
    //
    // ```text
    //   normaler Blit -> ...00     RGBA_MIN -> ...00     RGBA_SUB -> ...00
    //   RGBA_MAX      -> ...94     RGBA_ADD -> ...94
    // ```
    //
    // (MIN und SUB ergeben aus dem Ziel-Byte 0 eben wieder 0.) Deshalb
    // steht hier keine pauschale Maske mehr - die Entscheidung faellt je
    // Mischart weiter unten.
    let roh_alpha = !info.ziel_hat_alpha;
    // Dieselbe Frage fuer Lesen und Schreiben: rechnet dieser Weg roh auf
    // dem vierten Byte weiter, oder geht er durch die Farbzuordnung?
    let liest_roh = matches!(
        info.weg,
        Blitweg::Misch(
            flags::BLEND_RGBA_ADD
                | flags::BLEND_RGBA_SUB
                | flags::BLEND_RGBA_MULT
                | flags::BLEND_RGBA_MIN
                | flags::BLEND_RGBA_MAX
                | flags::BLEND_PREMULTIPLIED
        )
    );
    let qb = quell_buf.pitch as usize;
    let qbpp = quell_buf.bpp as usize;
    let flaechen_alpha = info.flaechen_alpha;

    // Die Schluesselfarbe gilt **nur beim normalen Blit**: jede `BLEND_*`-
    // Mischart uebergeht sie (gemessen, alle zwoelf durchprobiert), und
    // `alphablit_alpha` kennt sie gar nicht - die Routine hat schlicht
    // keinen Test dafuer, obwohl der Selbstblit sie dorthin schicken kann.
    let ueberspringen = match info.weg {
        Blitweg::Misch(_)
        | Blitweg::PremultipliedRgbKopie
        | Blitweg::PygameAlpha
        | Blitweg::PygameSolid => None,
        // `alphablit_colorkey` ueberspringt nicht, sondern rechnet mit
        // Alpha 0 weiter - das ist nicht dasselbe, sobald das Ziel
        // durchsichtig ist (siehe `PygameAlphaBlend`).
        Blitweg::PygameColorkey => None,
        _ => info.colorkey,
    };

    // Schnellweg: reines Kopieren ohne Schluesselfarbe -> memcpy je Zeile.
    // Der Schnellweg gilt nur, wenn beide Seiten dasselbe Format haben.
    // Sonst ist "kopieren" in Wahrheit ein Umsetzen: eine Quelle ohne
    // Alphakanal wird auf dem Ziel undurchsichtig, eine Quelle mit
    // Alphakanal verliert ihr Alpha.
    // **Der Schnellweg braucht jetzt auch dasselbe Format.** Zwei
    // Flaechen koennen gleich viele Bytes je Pixel haben und trotzdem
    // verschieden gepackt sein - eine mit `(0xFF0000, 0xFF00, 0xFF, 0)`
    // und eine mit `(0xFF, 0xFF00, 0xFF0000, 0)` sind beide 32-bittig und
    // vertauschen Rot und Blau. Ein `copy_from_slice` daraus waere ein
    // Farbtausch ohne Fehlermeldung.
    let schnell = info.weg == Blitweg::Kopie
        && ueberspringen.is_none()
        && info.quelle_hat_alpha == info.ziel_hat_alpha
        && quell_buf.format == ziel_buf.format
        && quell_buf.tafel == ziel_buf.tafel;

    // Der 32-Bit-Weg: dieselbe Rechnung wie die Schleife unten, nur mit
    // allen Weichen **vor** der Schleife. Siehe `blit_32`.
    if !schnell
        && info.raster.is_none()
        && ist_standard32(quell_buf)
        && ist_standard32(ziel_buf)
    {
        blit_32(
            ziel_buf,
            ziel_off,
            ziel,
            quell_buf,
            quell_off,
            quell_start,
            info,
            Vorlauf {
                roh_alpha,
                liest_roh,
                ueberspringen,
            },
        );
        return;
    }

    for y in 0..ziel.h {
        let zi = (ziel_off.1 + (ziel.y + y) as usize) * zb
            + (ziel_off.0 + ziel.x as usize) * zbpp;
        let qi = (quell_off.1 + (quell_start.1 + y) as usize) * qb
            + (quell_off.0 + quell_start.0 as usize) * qbpp;

        if schnell {
            // Zwei verschiedene Puffer - der Selbstblit geht ueber eine
            // Zwischenkopie (`blit_selbst`) -, also darf aus dem einen
            // gelesen und in den anderen geschrieben werden, ohne die
            // Zeile erst in einen eigenen `Vec` zu legen.
            let spanne = ziel.w as usize * zbpp;
            let q: &[u8] = bytemuck::cast_slice(&quell_buf.data);
            let z: &mut [u8] = bytemuck::cast_slice_mut(&mut ziel_buf.data);
            z[zi..zi + spanne].copy_from_slice(&q[qi..qi + spanne]);
            continue;
        }

        for x in 0..ziel.w as usize {
            let s = quell_buf.nach_argb(quell_buf.lies(qi + x * qbpp));
            if let Some(k) = ueberspringen {
                // SDL vergleicht ohne den Alphakanal:
                // `rgbmask = ~srcfmt->Amask; ckey = info->colorkey & rgbmask;`
                if s & 0x00FF_FFFF == k & 0x00FF_FFFF {
                    continue;
                }
            }
            let d_roh = ziel_buf.nach_argb(ziel_buf.lies(zi + x * zbpp));
            // **Wie das Ziel gelesen wird, haengt am Weg.**
            //
            // pygames `GET_PIXELVALS` setzt `dA = 255`, wenn das Ziel
            // keinen Alphakanal hat (`!dstppa`) - die Mischroutinen sehen
            // dort also eine undurchsichtige Flaeche, egal was im
            // ungenutzten Byte steht. Die `BLEND_RGBA_*`-Mischarten und
            // `BLEND_PREMULTIPLIED` gehen daran vorbei und rechnen **roh**
            // auf dem Byte weiter. Am Rohspeicher gemessen, Ziel
            // `(72,197,142)` ohne Alphakanal, Quelle Alpha 148:
            //
            // ```text
            //   RGBA_ADD -> 94   RGBA_MAX -> 94   PREMULTIPLIED -> 94
            //   RGBA_SUB -> 00   RGBA_MIN -> 00   RGBA_MULT     -> 00
            //   normal, alle RGB_* -> 00
            // ```
            //
            // `SUB`, `MIN` und `MULT` ergeben aus einer Null eben wieder
            // Null - deshalb verraet erst `ADD` und `MAX`, dass dort
            // wirklich roh gerechnet wird.
            let d = if roh_alpha && !liest_roh {
                d_roh | 0xFF00_0000
            } else {
                d_roh
            };

            // Das wirksame Quell-Alpha: Pixel-Alpha mal Flaechen-Alpha
            // (`alphablit_alpha`: `sA = (sA * alpha) / 255;`).
            let pixel_alpha = if info.quelle_hat_alpha {
                (s >> 24) as u8
            } else {
                255
            };
            let wirksam = if flaechen_alpha == 255 {
                pixel_alpha
            } else if pixel_alpha == 255 {
                flaechen_alpha
            } else {
                ((pixel_alpha as u32 * flaechen_alpha as u32) / 255) as u8
            };

            let neu = match info.weg {
                // SDL kopiert die Farbe; das Ziel-Alpha kommt bei einer
                // Quelle ohne Alphakanal aus `info->a` (dem Flaechen-Alpha,
                // das auf diesem Weg immer 255 ist), sonst aus dem Pixel
                // (`BlitNtoNKeyCopyAlpha`, `SDL_BlitCopy`).
                Blitweg::Kopie => {
                    if info.quelle_hat_alpha {
                        s
                    } else {
                        (s & 0x00FF_FFFF) | ((flaechen_alpha as u32) << 24)
                    }
                }
                // Nur die drei Farbkanaele, der Alphakanal bleibt stehen.
                Blitweg::PremultipliedRgbKopie => (d & 0xFF00_0000) | (s & 0x00FF_FFFF),
                Blitweg::SdlFlaechenAlpha => blend::SdlFlaechenAlpha::apply(d, s, wirksam),
                Blitweg::SdlFlaechenAlphaKey => {
                    blend::SdlFlaechenAlphaKey::apply(d, s, wirksam)
                }
                Blitweg::SdlLangsam => blend::SdlLangsam::apply(d, s, wirksam),
                Blitweg::PygameAlpha | Blitweg::PygameSolid => {
                    // `alphablit_solid` mischt mit dem Flaechen-Alpha statt
                    // mit dem Pixel-Alpha - bei einer Quelle ohne
                    // Alphakanal ist `wirksam` genau das.
                    blend::PygameAlphaBlend::apply(d, s, wirksam)
                }
                Blitweg::PygameColorkey => {
                    // `sA = (pixel == colorkey) ? 0 : alpha;` - und zwar mit
                    // dem **ganzen** Pixel verglichen, nicht nur der Farbe.
                    let a = match info.colorkey {
                        Some(k) if s == k => 0,
                        _ => flaechen_alpha,
                    };
                    blend::PygameAlphaBlend::apply(d, s, a)
                }
                Blitweg::Misch(f) => {
                    // Die Quelle so, wie die RGBA-Mischarten sie sehen
                    // (siehe die Tabelle in `blit_ex`).
                    let sa = match info.rgba_quell_alpha {
                        None => s,
                        Some(a) => (s & 0x00FF_FFFF) | ((a as u32) << 24),
                    };
                    match f {
                        flags::BLEND_RGB_ADD => blend::RgbAdd::apply(d, s, wirksam),
                        flags::BLEND_RGB_SUB => blend::RgbSub::apply(d, s, wirksam),
                        flags::BLEND_RGB_MULT => blend::RgbMult::apply(d, s, wirksam),
                        flags::BLEND_RGB_MIN => blend::RgbMin::apply(d, s, wirksam),
                        flags::BLEND_RGB_MAX => blend::RgbMax::apply(d, s, wirksam),
                        flags::BLEND_RGBA_ADD => blend::RgbaAdd::apply(d, sa, wirksam),
                        flags::BLEND_RGBA_SUB => blend::RgbaSub::apply(d, sa, wirksam),
                        flags::BLEND_RGBA_MULT => blend::RgbaMult::apply(d, sa, wirksam),
                        flags::BLEND_RGBA_MIN => blend::RgbaMin::apply(d, sa, wirksam),
                        flags::BLEND_RGBA_MAX => blend::RgbaMax::apply(d, sa, wirksam),
                        flags::BLEND_PREMULTIPLIED => {
                            blend::Premultiplied::apply(d, sa, wirksam)
                        }
                        // **Der eigene Weg von SDL**, nicht der von pygame.
                        //
                        // Hier fehlte der Zweig ganz, und der `_`-Fall
                        // unten hat ihn stillschweigend an pygames
                        // Alphamischer weitergegeben. Das ist die
                        // unangenehme Sorte Lücke: `BLEND_ALPHA_SDL2` gab
                        // eine Antwort, die nach einer Antwort aussah.
                        // Gemessen lag sie um bis zu 255 daneben - bei
                        // Quell-Alpha 0 kam die Quelle heraus, wo das Ziel
                        // haette stehenbleiben muessen.
                        //
                        // `surface.c` sagt es ausdruecklich: die Kette
                        // `if ((blend_flags != 0 && blend_flags !=
                        // PYGAME_BLEND_ALPHA_SDL2) || ...)` nimmt diese
                        // eine Flagge von pygames Weg **aus**. Sie heisst
                        // nicht zufaellig so.
                        flags::BLEND_ALPHA_SDL2 => {
                            blend::SdlPixelAlpha::apply(d, s, wirksam)
                        }
                        _ => blend::PygameAlphaBlend::apply(d, s, wirksam),
                    }
                }
            };
            // Und beim Schreiben dasselbe: was nicht roh rechnet, geht
            // durch die Zuordnung - und die laesst das Alpha fallen.
            //
            // **Mit einer Ausnahme.** `BlitRGBtoRGBSurfaceAlpha` endet mit
            // `*dstp = d1 | d | 0xff000000` und setzt das ungenutzte
            // vierte Byte des Ziels damit auf `0xFF`. Am Rohspeicher
            // nachgemessen, Ziel ohne Alphakanal, Quelle ohne Alphakanal:
            //
            // ```text
            //   set_alpha nie gerufen -> ...00   (reiner Kopierblitter)
            //   set_alpha(96)         -> ...ff   (dieser Weg hier)
            //   set_alpha(96) + Schluesselfarbe -> ...00  (anderer Weg)
            //   set_alpha(255)        -> ...00   (wieder der Kopierblitter)
            // ```
            //
            // Sichtbar ist das nie - aber `flood_fill` und `scale2x`
            // vergleichen rohe Woerter und merken es.
            // Dasselbe wie in `fill_ex`: der Wert muss in das Format des
            // **Ziels** passen.
            //
            // Ohne Ausnahme, auch beim reinen Kopieren: dort steht zwar
            // schon ein gueltiger Wert, aber nur, wenn die Quelle dasselbe
            // Format hat - und das steht hier nicht fest. Ein zweites
            // Einrasten auf einen bereits eingerasteten Wert aendert
            // nichts, `naechster_eintrag` und `quantisieren` sind beide
            // idempotent.
            let neu = match &info.raster {
                Some(r) => r.an(neu),
                None => neu,
            };
            let fertig = if roh_alpha {
                if info.weg == Blitweg::SdlFlaechenAlpha {
                    neu | 0xFF00_0000
                } else if liest_roh {
                    neu
                } else {
                    neu & 0x00FF_FFFF
                }
            } else {
                neu
            };
            let wort = ziel_buf.von_argb(fertig);
            ziel_buf.setz(zi + x * zbpp, wort);
        }
    }
}

#[cfg(test)]
mod tests {
    /// Vier Zahlen - Masken, Shifts oder Losses.
    type Vier = (u32, u32, u32, u32);

    use super::*;

    #[test]
    fn neue_flaeche_ist_schwarz_bzw_durchsichtig() {
        let a = Surface::new((4, 3));
        assert_eq!(a.get_size(), (4, 3));
        assert_eq!(a.get_at((0, 0)), Color::rgba(0, 0, 0, 255));
        let b = Surface::with_flags((4, 3), SRCALPHA);
        assert_eq!(b.get_at((0, 0)), Color::rgba(0, 0, 0, 0));
        assert!(b.has_alpha());
    }

    #[test]
    fn fuellen_und_auslesen() {
        let mut s = Surface::new((4, 4));
        s.fill((10, 20, 30));
        assert_eq!(s.get_at((2, 2)), Color::rgba(10, 20, 30, 255));
        s.fill_rect((1, 2, 3), Rect::new(0, 0, 2, 2));
        assert_eq!(s.get_at((0, 0)), Color::rgba(1, 2, 3, 255));
        assert_eq!(s.get_at((3, 3)), Color::rgba(10, 20, 30, 255));
    }

    #[test]
    fn alpha_blit_trifft_die_messwerte() {
        // Gemessen an echtem pygame: (100,150,200) + (50,60,70,128)
        // ergibt (75,105,135).
        let mut dst = Surface::new((2, 2));
        dst.fill((100, 150, 200));
        let mut src = Surface::with_flags((2, 2), SRCALPHA);
        src.fill((50, 60, 70, 128));
        dst.blit(&src, (0, 0));
        assert_eq!(dst.get_at((0, 0)).as_tuple(), (75, 105, 135, 255));
    }

    #[test]
    fn flaechen_alpha_multipliziert_sich_mit_dem_pixel_alpha() {
        // Gemessen: src-Alpha 128 mal set_alpha(128) wirkt wie 64.
        // (200,100,50) auf Schwarz mit wirksamem Alpha 64 -> 50,25,12
        let mut dst = Surface::new((2, 2));
        dst.fill((0, 0, 0));
        let mut src = Surface::with_flags((2, 2), SRCALPHA);
        src.fill((200, 100, 50, 128));
        src.set_alpha(Some(128));
        dst.blit(&src, (0, 0));
        assert_eq!(dst.get_at((0, 0)).as_tuple(), (50, 25, 12, 255));
    }

    #[test]
    fn clip_begrenzt_das_zeichnen() {
        let mut s = Surface::new((10, 10));
        s.fill((0, 0, 0));
        s.set_clip(Some(Rect::new(2, 2, 3, 3)));
        s.fill((255, 255, 255));
        assert_eq!(s.get_at((1, 1)).r, 0, "ausserhalb bleibt schwarz");
        assert_eq!(s.get_at((2, 2)).r, 255);
        assert_eq!(s.get_at((4, 4)).r, 255);
        assert_eq!(s.get_at((5, 5)).r, 0);
        s.set_clip(None);
        assert_eq!(s.get_clip(), s.get_rect());
    }

    #[test]
    fn teilflaeche_teilt_den_speicher() {
        let mut s = Surface::new((10, 10));
        s.fill((0, 0, 0));
        let mut teil = s.subsurface(Rect::new(2, 2, 4, 4));
        teil.fill((255, 0, 0));
        // Das Original muss sich mitveraendert haben.
        assert_eq!(s.get_at((3, 3)).r, 255, "subsurface teilt den Speicher");
        assert_eq!(s.get_at((1, 1)).r, 0);
        assert_eq!(teil.get_abs_offset(), (2, 2));
        assert!(teil.get_parent().is_some());
    }

    #[test]
    fn kopie_teilt_nichts() {
        let mut s = Surface::new((4, 4));
        s.fill((1, 2, 3));
        let mut k = s.copy();
        k.fill((9, 9, 9));
        assert_eq!(s.get_at((0, 0)).r, 1, "die Kopie ist eigenstaendig");
        assert_eq!(k.get_at((0, 0)).r, 9);
    }

    #[test]
    fn colorkey_wird_uebersprungen() {
        let mut dst = Surface::new((2, 2));
        dst.fill((0, 0, 0));
        let mut src = Surface::new((2, 2));
        src.fill((255, 0, 255));
        src.set_colorkey(Some((255u8, 0u8, 255u8)));
        dst.blit(&src, (0, 0));
        assert_eq!(dst.get_at((0, 0)).r, 0, "die Schluesselfarbe bleibt aussen vor");
    }

    /// Ein Blit, wie ihn die Auswahlkette sieht: Ziel `(100,150,200,200)`,
    /// Quelle `(50,60,70,90)`.
    fn probeblit(
        ziel_mit_alpha: bool,
        quelle_mit_alpha: bool,
        setze_alpha: Option<Option<u8>>,
        schluesselfarbe: Option<(u8, u8, u8)>,
    ) -> (u8, u8, u8, u8) {
        let mut dst = Surface::with_flags((2, 2), if ziel_mit_alpha { SRCALPHA } else { 0 });
        dst.fill((100, 150, 200, 200));
        let mut src = Surface::with_flags((2, 2), if quelle_mit_alpha { SRCALPHA } else { 0 });
        src.fill((50, 60, 70, 90));
        if let Some(a) = setze_alpha {
            src.set_alpha(a);
        }
        if let Some(k) = schluesselfarbe {
            src.set_colorkey(Some(k));
        }
        dst.blit(&src, (0, 0));
        dst.get_at((0, 0)).as_tuple()
    }

    /// **Der Kern dieser Datei:** jede Zeile der Auswahlkette einmal
    /// durchgespielt. Alle Sollwerte sind an echtem pygame-ce 2.5.7
    /// gemessen (`SDL_VIDEODRIVER=dummy`), nicht gerechnet.
    ///
    /// Die Zeilen "Weg 1" und "Weg 2" unterscheiden sich nur um eins pro
    /// Kanal - genau daran haengt, ob die Kette richtig verzweigt.
    #[test]
    fn die_auswahlkette_trifft_die_messwerte() {
        // Ohne Mischmodus kopiert SDL, auch mit set_alpha(255).
        assert_eq!(probeblit(true, false, None, None), (50, 60, 70, 255));
        assert_eq!(
            probeblit(true, false, Some(Some(255)), None),
            (50, 60, 70, 255),
            "set_alpha(255) auf einer Flaeche ohne Alphakanal kopiert exakt"
        );

        // Weg 3: BlitRGBtoRGBSurfaceAlpha, Ziel-Alpha wird hart 255.
        assert_eq!(
            probeblit(true, false, Some(Some(128)), None),
            (75, 105, 135, 255)
        );
        assert_eq!(
            probeblit(false, false, Some(Some(128)), None),
            (75, 105, 135, 255)
        );

        // Weg 4: dieselben Farben, aber das Ziel-Alpha wird gerechnet.
        assert_eq!(
            probeblit(true, false, Some(Some(128)), Some((1, 2, 3))),
            (75, 105, 135, 228),
            "BlitNtoNSurfaceAlphaKey rechnet das Ziel-Alpha mit"
        );

        // Weg 1: pygames eigener Blit fuer Quellen mit Alphakanal.
        assert_eq!(probeblit(true, true, None, None), (82, 118, 154, 220));
        assert_eq!(
            probeblit(true, true, Some(Some(128)), None),
            (91, 134, 177, 210),
            "Pixel-Alpha mal Flaechen-Alpha"
        );

        // set_alpha(None) nimmt den Mischmodus weg - dann wird kopiert,
        // samt Alphakanal.
        assert_eq!(
            probeblit(true, true, Some(None), None),
            (50, 60, 70, 90),
            "ohne Mischmodus kopiert SDL die Quelle mitsamt Alpha"
        );

        // Weg 2: Alphakanal **und** Schluesselfarbe -> SDL_Blit_Slow.
        // Ein Kanal Unterschied zu Weg 1, und der ist messbar.
        assert_eq!(probeblit(true, true, None, Some((1, 2, 3))), (81, 118, 153, 219));
        assert_eq!(
            probeblit(true, true, Some(Some(128)), Some((1, 2, 3))),
            (90, 133, 176, 209)
        );
    }

    /// Auf ein **voellig durchsichtiges** Ziel wird die Quelle uebernommen
    /// statt hineingemischt - der `else`-Zweig von `ALPHA_BLEND`. Ohne ihn
    /// weicht jeder Blit auf eine frische `SRCALPHA`-Flaeche ab.
    #[test]
    fn auf_durchsichtiges_ziel_wird_uebernommen() {
        let mut dst = Surface::with_flags((2, 2), SRCALPHA);
        dst.fill((100, 150, 200, 0));
        let mut src = Surface::with_flags((2, 2), SRCALPHA);
        src.fill((50, 60, 70, 90));
        dst.blit(&src, (0, 0));
        assert_eq!(dst.get_at((0, 0)).as_tuple(), (50, 60, 70, 90));

        // Mit Flaechen-Alpha wird das Quell-Alpha vorher heruntergerechnet,
        // die Farbe aber weiterhin uebernommen.
        let mut dst = Surface::with_flags((2, 2), SRCALPHA);
        dst.fill((100, 150, 200, 0));
        src.set_alpha(Some(128));
        dst.blit(&src, (0, 0));
        assert_eq!(dst.get_at((0, 0)).as_tuple(), (50, 60, 70, 45));

        // SDLs Weg kennt diesen Zweig nicht: Quelle ohne Alphakanal.
        let mut dst = Surface::with_flags((2, 2), SRCALPHA);
        dst.fill((100, 150, 200, 0));
        let mut src = Surface::new((2, 2));
        src.fill((50, 60, 70));
        src.set_alpha(Some(128));
        dst.blit(&src, (0, 0));
        assert_eq!(dst.get_at((0, 0)).as_tuple(), (75, 105, 135, 255));
    }

    /// Eine Schluesselfarbe, die wirklich trifft, laesst das Ziel in Ruhe -
    /// auch auf SDLs Flaechen-Alpha-Weg.
    #[test]
    fn treffende_schluesselfarbe_laesst_das_ziel_stehen() {
        assert_eq!(
            probeblit(true, false, Some(Some(128)), Some((50, 60, 70))),
            (100, 150, 200, 200)
        );
    }

    /// Blittet eine 4x1-Flaeche `[200, 200, 10, 10]` auf sich selbst und
    /// gibt den roten Kanal zurueck.
    fn selbstblit(mit_alpha: bool, setze: Option<u8>, ck: Option<(u8, u8, u8)>, ziel_x: i32,
                  breite: i32) -> Vec<u8> {
        let mut s = Surface::with_flags((4, 1), if mit_alpha { SRCALPHA } else { 0 });
        for (x, v) in [200u8, 200, 10, 10].iter().enumerate() {
            s.set_at((x as i32, 0), (*v, *v, *v, 255u8));
        }
        if let Some(a) = setze {
            s.set_alpha(Some(a));
        }
        if let Some(k) = ck {
            s.set_colorkey(Some(k));
        }
        let kopie = s.clone(); // teilt denselben Puffer
        s.blit_ex(&kopie, (ziel_x, 0), Some(Rect::new(0, 0, breite, 1)), 0);
        (0..4).map(|x| s.get_at((x, 0)).r).collect()
    }

    /// Der Selbstblit ist der **einzige** Weg zu `alphablit_solid` und
    /// `alphablit_colorkey`: nur wenn eine Flaeche sich ueberlappend selbst
    /// beblittet, macht pygame den Blit ueberhaupt selbst
    /// (`pgSurface_Blit`, erste Bedingung). Alle Werte gemessen.
    ///
    /// **Dieser Test war gruen, waehrend die Regel fuer jeden Aufruf aus
    /// Python falsch war** - und zwar ohne Luecke im Test: `selbstblit`
    /// uebergibt immer einen Bereich, und die dritte Bedingung stand hier
    /// als `area.is_some()`. Wo ein Bereich mitkommt, sind die falsche und
    /// die richtige Frage nicht zu unterscheiden. `Surface.blit(s, ziel)`
    /// ohne `area` fiel dagegen durch - siehe
    /// `selbstblit_mit_flaechenalpha_nur_bei_ueberlappung_wie_pygame`, das
    /// den Fall ohne Bereich prueft. Ein Test, der eine Bedingung nur in
    /// **einer** Gestalt vorfuehrt, prueft sie nicht.
    #[test]
    fn ueberlappender_selbstblit_nimmt_pygames_eigenen_weg() {
        // Ziel (1,0), Bereich (0,0,3,1): die Bereiche ueberlappen.
        // `alphablit_solid` rechnet mit pygames Formel - 11, nicht 10.
        assert_eq!(selbstblit(false, Some(1), None, 1, 3), vec![200, 200, 11, 10]);

        // `alphablit_colorkey` setzt das Alpha der Schluesselpixel auf 0,
        // statt sie zu ueberspringen - hier bleibt das Ziel stehen.
        assert_eq!(
            selbstblit(false, Some(1), Some((200, 200, 200)), 1, 3),
            vec![200, 200, 10, 10]
        );

        // `alphablit_alpha` **kennt keine Schluesselfarbe**: dieselbe
        // Flaeche mit Alphakanal uebergeht sie und mischt normal.
        assert_eq!(
            selbstblit(true, Some(1), Some((200, 200, 200)), 1, 3),
            vec![200, 200, 11, 10],
            "die Routine fuer Quellen mit Alphakanal hat gar keinen Test dafuer"
        );

        // Ohne Ueberlappung (Ziel (2,0), Bereich (0,0,2,1)) geht derselbe
        // Blit an SDL - und SDLs Flaechen-Alpha rundet anders: 10 statt 11.
        assert_eq!(selbstblit(false, Some(1), None, 2, 2), vec![200, 200, 10, 10]);
    }

    #[test]
    fn blit_gibt_das_beschriebene_rechteck_zurueck() {
        let mut dst = Surface::new((10, 10));
        let src = Surface::new((4, 4));
        assert_eq!(dst.blit(&src, (2, 3)), Rect::new(2, 3, 4, 4));
        // Teilweise ausserhalb -> zugeschnitten.
        assert_eq!(dst.blit(&src, (8, 8)), Rect::new(8, 8, 2, 2));
        // Ganz ausserhalb -> leer.
        assert!(dst.blit(&src, (50, 50)).is_empty());
    }

    #[test]
    fn blit_auf_sich_selbst_verschmiert_nicht() {
        let mut s = Surface::new((8, 1));
        for x in 0..8 {
            s.set_at((x, 0), (x as u8 * 10, 0, 0));
        }
        let kopie = s.clone();
        // Um zwei nach rechts schieben - ueberlappend.
        s.blit_area(&kopie, (2, 0), Rect::new(0, 0, 6, 1));
        assert_eq!(s.get_at((2, 0)).r, 0);
        assert_eq!(s.get_at((3, 0)).r, 10);
        assert_eq!(s.get_at((7, 0)).r, 50);
    }

    #[test]
    fn umgebendes_rechteck_findet_den_inhalt() {
        let mut s = Surface::with_flags((10, 10), SRCALPHA);
        s.fill_rect((255, 0, 0, 255), Rect::new(3, 4, 2, 2));
        assert_eq!(s.get_bounding_rect(1), Rect::new(3, 4, 2, 2));
        let leer = Surface::with_flags((4, 4), SRCALPHA);
        assert!(leer.get_bounding_rect(1).is_empty());

        // Gemessen an pygame: `min_alpha` vergleicht mit **>=**, und die
        // Schwelle 5 laesst einen Pixel mit Alpha 5 noch durch.
        let mut fein = Surface::with_flags((6, 4), SRCALPHA);
        fein.fill((0, 0, 0, 0));
        fein.set_at((2, 1), (9, 9, 9, 5));
        fein.set_at((4, 3), (9, 9, 9, 200));
        assert_eq!(fein.get_bounding_rect(5), Rect::new(2, 1, 3, 3));
        assert_eq!(fein.get_bounding_rect(6), Rect::new(4, 3, 1, 1));
        assert_eq!(fein.get_bounding_rect(200), Rect::new(4, 3, 1, 1));
        assert!(fein.get_bounding_rect(201).is_empty());

        // **Ohne Alphakanal immer die ganze Flaeche** - auch bei 255.
        let ohne = Surface::new((6, 4));
        assert_eq!(ohne.get_bounding_rect(1), Rect::new(0, 0, 6, 4));
        assert_eq!(ohne.get_bounding_rect(255), Rect::new(0, 0, 6, 4));
    }

    /// Ohne Alphakanal verwirft die Flaeche das Alpha - auf **jedem** Weg.
    ///
    /// Die Sollwerte stammen aus einer Messung an echtem pygame-ce 2.5.7:
    /// eine `Surface((4,4))` ohne `SRCALPHA`, beschrieben mit der Farbe
    /// `(255, 0, 0, 128)`, liefert bei `get_at` durchweg Alpha 255. Kein
    /// einziger Referenzfall deckt das ab, weil dort alle Farben ohnehin
    /// undurchsichtig sind - deshalb steht es hier.
    #[test]
    fn ohne_alphakanal_bleibt_alles_undurchsichtig() {
        let mut ohne = Surface::new((4, 4));
        ohne.fill((255, 0, 0, 128));
        assert_eq!(ohne.get_at((0, 0)), Color::rgba(255, 0, 0, 255), "fill");

        assert!(ohne.set_at((1, 1), (0, 255, 0, 77)));
        assert_eq!(ohne.get_at((1, 1)), Color::rgba(0, 255, 0, 255), "set_at");

        // Und beim Blitten: der Wert 64 mischt mit, wird aber nicht abgelegt.
        let mut q = Surface::with_flags((2, 2), SRCALPHA);
        q.fill((9, 9, 9, 64));
        ohne.blit(&q, (0, 2));
        assert_eq!(ohne.get_at((0, 2)).a, 255, "blit legt kein Alpha ab");

        // Mit Alphakanal bleibt dagegen alles erhalten.
        let mut mit = Surface::with_flags((4, 4), SRCALPHA);
        mit.fill((255, 0, 0, 128));
        assert_eq!(mit.get_at((0, 0)), Color::rgba(255, 0, 0, 128));
        mit.set_at((1, 1), (0, 255, 0, 77));
        assert_eq!(mit.get_at((1, 1)), Color::rgba(0, 255, 0, 77));
    }

    #[test]
    fn set_at_meldet_ob_geschrieben_wurde() {
        let mut s = Surface::new((6, 6));
        assert!(s.set_at((0, 0), (1, 2, 3)));
        assert!(!s.set_at((-1, 0), (1, 2, 3)), "links daneben");
        assert!(!s.set_at((6, 0), (1, 2, 3)), "rechts daneben");

        s.set_clip(Some(Rect::new(2, 2, 2, 2)));
        assert!(s.set_at((2, 2), (1, 2, 3)));
        assert!(!s.set_at((1, 2), (1, 2, 3)), "vom Clip ausgeschlossen");
    }

    /// Das Pixelformat - alle Werte an echtem pygame-ce gemessen.
    #[test]
    fn das_pixelformat_meldet_pygames_werte() {
        let ohne = Surface::new((7, 5));
        let mit = Surface::with_flags((7, 5), SRCALPHA);

        for s in [&ohne, &mit] {
            assert_eq!(s.get_bitsize(), 32);
            assert_eq!(s.get_bytesize(), 4);
            assert_eq!(s.get_pitch(), 28, "7 Pixel mal 4 Byte");
        }

        assert_eq!(ohne.get_masks(), (0x00FF_0000, 0xFF00, 0xFF, 0));
        assert_eq!(mit.get_masks(), (0x00FF_0000, 0xFF00, 0xFF, 0xFF00_0000));
        assert_eq!(ohne.get_shifts(), (16, 8, 0, 0));
        assert_eq!(mit.get_shifts(), (16, 8, 0, 24));
        assert_eq!(ohne.get_losses(), (0, 0, 0, 8), "ohne Alphakanal fehlen 8 Bit");
        assert_eq!(mit.get_losses(), (0, 0, 0, 0));
    }

    /// `map_rgb` und der rohe Pixelwert - hier wird das ungenutzte vierte
    /// Byte ueberhaupt erst sichtbar.
    #[test]
    fn map_rgb_laesst_das_alpha_fallen() {
        let mut ohne = Surface::new((4, 4));
        let mut mit = Surface::with_flags((4, 4), SRCALPHA);

        // Gemessen: 0x00010203, **nicht** 0x04010203.
        assert_eq!(ohne.map_rgb((1u8, 2u8, 3u8, 4u8)), 0x0001_0203);
        assert_eq!(mit.map_rgb((1u8, 2u8, 3u8, 4u8)), 0x0401_0203);
        // Ohne Alpha angegeben gilt 255.
        assert_eq!(mit.map_rgb((1u8, 2u8, 3u8)), 0xFF01_0203);

        // Der Rueckweg meldet die Farbe so, wie man sie sieht.
        assert_eq!(ohne.unmap_rgb(0x0001_0203), Color::rgba(1, 2, 3, 255));
        assert_eq!(mit.unmap_rgb(0x0401_0203), Color::rgba(1, 2, 3, 4));

        ohne.set_at((0, 0), (9, 8, 7, 6));
        mit.set_at((0, 0), (9, 8, 7, 6));
        // pygame: 0x00090807 - die 6 ist weg, weil es keinen Kanal gibt.
        assert_eq!(ohne.get_at_mapped((0, 0)), 0x0009_0807);
        assert_eq!(mit.get_at_mapped((0, 0)), 0x0609_0807);
        // Sichtbar ist trotzdem beides undurchsichtig bzw. mit Alpha 6.
        assert_eq!(ohne.get_at((0, 0)).a, 255);
        assert_eq!(mit.get_at((0, 0)).a, 6);
    }

    #[test]
    fn premul_alpha_braucht_einen_alphakanal() {
        let mut s = Surface::with_flags((2, 2), SRCALPHA);
        s.set_at((0, 0), (200, 100, 50, 128));
        let v = s.premul_alpha().unwrap();
        // Gemessen an pygame: (100, 50, 25, 128).
        assert_eq!(v.get_at((0, 0)), Color::rgba(100, 50, 25, 128));

        // Und ohne Alphakanal geht es nicht - wortgleich wie in pygame.
        assert_eq!(
            Surface::new((2, 2)).premul_alpha().unwrap_err(),
            "source surface to be alpha pre-multiplied must have alpha channel"
        );
    }

    /// Die drei Nachzuegler - Sollwerte an echtem pygame gemessen.
    /// Der leere Rueckgabewert sieht je nach Weg anders aus - gemessen.
    #[test]
    fn ein_weggeschnittener_blit_meldet_je_nach_weg_anderes() {
        let mut z = Surface::new((48, 48));
        let undurchsichtig = Surface::new((6, 3));
        let mut mit_alpha = Surface::with_flags((6, 3), SRCALPHA);
        mit_alpha.fill((1, 2, 3, 255));
        let mut flaechen_alpha = Surface::new((6, 3));
        flaechen_alpha.set_alpha(Some(128));

        // SDLs Wege lassen die negative Koordinate stehen.
        assert_eq!(z.blit(&undurchsichtig, (18, -5)), Rect::new(18, -5, 0, 0));
        assert_eq!(z.blit(&undurchsichtig, (-9, 18)), Rect::new(-9, 18, 0, 0));
        assert_eq!(z.blit(&flaechen_alpha, (18, -5)), Rect::new(18, -5, 0, 0));

        // pygames eigener Weg klemmt sie auf null - aber nur die
        // negative Seite, nicht den Ueberstand nach rechts.
        assert_eq!(z.blit(&mit_alpha, (18, -5)), Rect::new(18, 0, 0, 0));
        assert_eq!(z.blit(&mit_alpha, (-9, 18)), Rect::new(0, 18, 0, 0));
        assert_eq!(z.blit(&mit_alpha, (99, 18)), Rect::new(99, 18, 0, 0));
    }

    #[test]
    fn mischmodus_frect_und_fblits() {
        let mut ohne = Surface::new((7, 5));
        let mut mit = Surface::with_flags((7, 5), SRCALPHA);
        assert_eq!(ohne.get_blendmode(), 0);
        assert_eq!(mit.get_blendmode(), 1);
        ohne.set_alpha(Some(128));
        assert_eq!(ohne.get_blendmode(), 1);
        ohne.set_alpha(None);
        assert_eq!(ohne.get_blendmode(), 0);
        // Auch eine Flaeche **mit** Alphakanal verliert ihn.
        mit.set_alpha(None);
        assert_eq!(mit.get_blendmode(), 0);

        assert_eq!(
            ohne.get_frect(),
            crate::frect::FRect::new(0.0, 0.0, 7.0, 5.0)
        );
        // `get_frect(center=(1.5, 2.5))` ergibt in pygame (-2.0, 0.0, 7, 5).
        assert_eq!(
            ohne.get_frect().with_center((1.5, 2.5)),
            crate::frect::FRect::new(-2.0, 0.0, 7.0, 5.0)
        );

        // `fblits` schneidet das Ziel ab: (1.5, 1.5) landet auf (1, 1).
        let mut z = Surface::new((10, 10));
        let mut q = Surface::new((2, 2));
        q.fill((255, 0, 0));
        z.fblits(&[(&q, (1.5, 1.5)), (&q, (4.0, 4.0))], 0);
        assert_eq!(z.get_at((1, 1)).r, 255);
        assert_eq!(z.get_at((2, 2)).r, 255);
        assert_eq!(z.get_at((4, 4)).r, 255);
        assert_eq!(z.get_at((0, 0)).r, 0);
    }

    #[test]
    fn get_abs_parent_findet_die_wurzel() {
        let s = Surface::new((8, 8));
        let t = s.subsurface(Rect::new(1, 1, 5, 5));
        let u = t.subsurface(Rect::new(1, 1, 3, 3));
        assert_eq!(u.get_abs_parent().get_size(), (8, 8));
        assert_eq!(u.get_abs_offset(), (2, 2));
        assert_eq!(u.get_offset(), (1, 1), "nur gegenueber dem Elternteil");
    }

    #[test]
    fn wie_uebernimmt_die_eigenschaften_der_vorlage() {
        let mut vorlage = Surface::with_flags((8, 8), SRCALPHA);
        vorlage.set_alpha(Some(64));
        vorlage.set_colorkey(Some((1, 2, 3)));

        let neu = Surface::wie(&vorlage, (3, 5));
        assert_eq!(neu.get_size(), (3, 5), "aber eigene Groesse");
        assert!(neu.has_alpha());
        assert_eq!(neu.get_alpha(), Some(64));
        assert_eq!(neu.get_colorkey(), Some(Color::rgba(1, 2, 3, 255)));
    }

    #[test]
    fn mit_pixeln_trifft_auch_in_einer_subsurface() {
        // `s` braucht kein `mut`: geschrieben wird ausschliesslich durch
        // die Teilflaeche - und genau das ist der Punkt des Tests.
        let s = Surface::new((8, 8));
        let mut teil = s.subsurface(Rect::new(2, 3, 4, 2));
        teil.mit_pixeln(|daten, breite, (ox, oy)| {
            for y in 0..2 {
                for x in 0..4 {
                    daten[(oy + y) * breite + ox + x] = 0xFF00_FF00;
                }
            }
        });
        // Im Elternteil muss es an der richtigen Stelle stehen.
        assert_eq!(s.get_at((2, 3)), Color::rgba(0, 255, 0, 255));
        assert_eq!(s.get_at((5, 4)), Color::rgba(0, 255, 0, 255));
        assert_eq!(s.get_at((1, 3)), Color::rgba(0, 0, 0, 255), "daneben");
        assert_eq!(s.get_at((6, 4)), Color::rgba(0, 0, 0, 255), "daneben");
    }

    // ----- Das Pixelformat ------------------------------------------------
    //
    // Alle Erwartungen hier sind an echtem pygame-ce 2.5.7 gemessen, nicht
    // aus den Masken hergeleitet - siehe [`Format`].

    #[test]
    fn standardformate_stimmen_mit_pygame() {
        // Tiefe -> (bytesize, masks, shifts, losses)
        /// Tiefe, Bytes je Pixel, Masken, Shifts, Losses.
        type Fall = (u32, u32, Vier, Vier, Vier);
        let faelle: &[Fall] = &[
            (8, 1, (0, 0, 0, 0), (0, 0, 0, 0), (8, 8, 8, 8)),
            (12, 2, (0x0F00, 0x00F0, 0x000F, 0), (8, 4, 0, 0), (4, 4, 4, 8)),
            (15, 2, (0x7C00, 0x03E0, 0x001F, 0), (10, 5, 0, 0), (3, 3, 3, 8)),
            (16, 2, (0xF800, 0x07E0, 0x001F, 0), (11, 5, 0, 0), (3, 2, 3, 8)),
            (24, 3, (0x00FF_0000, 0x0000_FF00, 0x0000_00FF, 0), (16, 8, 0, 0), (0, 0, 0, 8)),
            (32, 4, (0x00FF_0000, 0x0000_FF00, 0x0000_00FF, 0), (16, 8, 0, 0), (0, 0, 0, 8)),
        ];
        for &(tiefe, bytes, masks, shifts, losses) in faelle {
            let f = Format::standard(tiefe, false).unwrap();
            assert_eq!(f.bytesize(), bytes, "bytesize bei {tiefe}");
            assert_eq!(f.masks, masks, "masks bei {tiefe}");
            assert_eq!(f.shifts(), shifts, "shifts bei {tiefe}");
            assert_eq!(f.losses(), losses, "losses bei {tiefe}");
        }
        // Mit Alphakanal gibt es nur zwei Standardformate.
        assert_eq!(
            Format::standard(16, true).unwrap().masks,
            (0x0F00, 0x00F0, 0x000F, 0xF000)
        );
        assert_eq!(Format::standard(32, true).unwrap(), Format::ARGB8888);
        for tiefe in [8, 12, 15, 24] {
            assert_eq!(
                Format::standard(tiefe, true).unwrap_err(),
                "no standard masks exist for given bitdepth with alpha"
            );
        }
        for tiefe in [0, 1, 48, 64] {
            assert_eq!(
                Format::standard(tiefe, false).unwrap_err(),
                "nonstandard bit depth given"
            );
        }
    }

    #[test]
    fn sechzehn_bit_verliert_bits_beim_ablegen() {
        // Gemessen: `Surface((4,4), depth=16)` mit (76,76,76) gefuellt
        // liest sich als (74,76,74) zurueck - und mit (255,0,0) als
        // (255,0,0), weil Weiss Weiss bleiben muss.
        let mut s = Surface::with_format((2, 2), 0, Format::standard(16, false).unwrap());
        s.fill(Color::rgba(76, 76, 76, 255));
        assert_eq!(s.get_at((0, 0)), Color::rgba(74, 76, 74, 255));
        s.fill(Color::rgba(255, 0, 0, 255));
        assert_eq!(s.get_at((0, 0)), Color::rgba(255, 0, 0, 255));
        // Und der gepackte Wert ist RGB565.
        assert_eq!(s.map_rgb_gepackt(Color::rgba(255, 0, 0, 255)), 0xF800);
        assert_eq!(
            s.unmap_rgb_gepackt(0xF800),
            Color::rgba(255, 0, 0, 255)
        );
    }

    #[test]
    fn zweiunddreissig_bit_bleibt_unberuehrt() {
        // Der geprueste Weg darf sich nicht bewegt haben: bei 24 und 32
        // Bit ist die Quantisierung die Identitaet.
        for tiefe in [24, 32] {
            let f = Format::standard(tiefe, false).unwrap();
            assert!(f.verlustfrei(), "bei {tiefe}");
            for v in [0x0001_0203u32, 0x00FF_FFFF, 0x0012_3456] {
                assert_eq!(f.quantisieren(v) & 0x00FF_FFFF, v, "bei {tiefe}");
            }
        }
        assert!(Format::ARGB8888.verlustfrei());
        assert!(!Format::standard(16, false).unwrap().verlustfrei());
    }

    #[test]
    fn die_vorbesetzte_farbtafel_ist_die_von_sdl_1_2() {
        let t = palette_vorgabe();
        assert_eq!(t.len(), 256);
        // Einzelwerte aus `Surface((4,4), depth=8).get_palette()`.
        for &(i, r, g, b) in &[
            (0usize, 0u8, 0u8, 0u8),
            (1, 0, 0, 85),
            (2, 0, 0, 170),
            (4, 0, 36, 0),
            (8, 0, 73, 0),
            (32, 85, 0, 0),
            (64, 170, 0, 0),
            (96, 255, 0, 0),
            (127, 255, 255, 255),
            (128, 0, 0, 0),
            (250, 255, 219, 170),
            (252, 255, 255, 0),
            (255, 255, 255, 255),
        ] {
            assert_eq!(t[i], Color::rgba(r, g, b, 255), "Eintrag {i}");
        }
        // Nur 128 verschiedene Farben, die zweite Haelfte spiegelt.
        for i in 128..256 {
            assert_eq!(t[i], t[i - 128], "Spiegelung bei {i}");
        }
    }

    #[test]
    fn eine_palettenflaeche_rastet_auf_ihre_tafel_ein() {
        // Gemessen an pygame: eine 8-Bit-Flaeche mit der Tafel
        // [rot, purpur, gold, blau, hellgruen] wird mit Rot gefuellt, und
        // `transform.invert` macht daraus Hellgruen - das Gegenteil von Rot
        // ist Cyan, und der naechste Tafeleintrag dazu ist Hellgruen.
        let mut s = Surface::with_format((2, 2), 0, Format::standard(8, false).unwrap());
        s.set_palette(Some(vec![
            Color::rgba(255, 0, 0, 255),
            Color::rgba(0, 0, 255, 255),
            Color::rgba(144, 238, 144, 255),
        ]));
        s.fill(Color::rgba(255, 0, 0, 255));
        assert_eq!(s.get_at((0, 0)), Color::rgba(255, 0, 0, 255));
        assert_eq!(s.map_rgb_gepackt(Color::rgba(0, 0, 255, 255)), 1, "Index");
        // Cyan gibt es nicht in der Tafel - Hellgruen liegt am naechsten.
        s.fill(Color::rgba(0, 255, 255, 255));
        assert_eq!(s.get_at((0, 0)), Color::rgba(144, 238, 144, 255));
    }

    #[test]
    fn format_und_tafel_ueberleben_kopie_und_teilflaeche() {
        // Ohne das kaeme aus `transform.scale` einer 8-Bit-Flaeche eine mit
        // 32 Bit - genau der pygame-Fehler 3463, den
        // `transform_test.test_scale__palette` hueten soll.
        let mut s = Surface::with_format((4, 4), 0, Format::standard(8, false).unwrap());
        s.set_palette(Some(palette_vorgabe()));
        for abgeleitet in [s.copy(), Surface::wie(&s, (8, 8)), s.subsurface(Rect::new(0, 0, 2, 2))] {
            assert_eq!(abgeleitet.get_bitsize(), 8);
            assert_eq!(abgeleitet.get_palette().map(|p| p.len()), Some(256));
        }
    }

    // ----- scroll: die drei Varianten, an echtem pygame abgelesen -------

    /// Baut die Probeflaeche: der rote Kanal traegt `x*10+1`, der gruene
    /// `y*10+1`. So verraet ein einzelner Wert, **woher** das Pixel kommt.
    fn scroll_flaeche() -> Surface {
        let mut s = Surface::new((4, 3));
        for y in 0..3i32 {
            for x in 0..4i32 {
                s.set_at((x, y), Color::rgba((x * 10 + 1) as u8, (y * 10 + 1) as u8, 7, 255));
            }
        }
        s
    }

    fn scroll_rot(s: &Surface) -> Vec<u8> {
        let mut aus = Vec::new();
        for y in 0..3i32 {
            for x in 0..4i32 {
                aus.push(s.get_at((x, y)).r);
            }
        }
        aus
    }

    /// `scroll(dx, dy, SCROLL_REPEAT)` - Werte aus echtem pygame-ce 2.5.7.
    #[test]
    fn scroll_wiederholend_wie_pygame() {
        #[allow(clippy::type_complexity)]
        let faelle: &[((i32, i32), Option<(i32, i32, i32, i32)>, [u8; 12])] = &[
            ((1, 0), None, [31, 1, 11, 21, 31, 1, 11, 21, 31, 1, 11, 21]),
            ((-1, 0), None, [11, 21, 31, 1, 11, 21, 31, 1, 11, 21, 31, 1]),
            ((0, 1), None, [1, 11, 21, 31, 1, 11, 21, 31, 1, 11, 21, 31]),
            ((2, 1), None, [21, 31, 1, 11, 21, 31, 1, 11, 21, 31, 1, 11]),
            // **Modulo**: ein Versatz breiter als die Flaeche verhaelt sich
            // wie der Rest. Genau hier unterscheidet sich diese Variante von
            // den anderen beiden.
            ((5, 0), None, [31, 1, 11, 21, 31, 1, 11, 21, 31, 1, 11, 21]),
            ((-5, 0), None, [11, 21, 31, 1, 11, 21, 31, 1, 11, 21, 31, 1]),
            // Herumlaufen **im Clip**, nicht ueber die ganze Flaeche.
            ((1, 0), Some((1, 0, 2, 3)), [1, 21, 11, 31, 1, 21, 11, 31, 1, 21, 11, 31]),
            ((1, 0), Some((0, 1, 4, 2)), [1, 11, 21, 31, 31, 1, 11, 21, 31, 1, 11, 21]),
        ];
        for &((dx, dy), clip, soll) in faelle {
            let mut s = scroll_flaeche();
            if let Some(c) = clip {
                s.set_clip(Some(Rect::new(c.0, c.1, c.2, c.3)));
            }
            s.scroll_wiederholend(dx, dy);
            assert_eq!(scroll_rot(&s), soll.to_vec(), "({dx}, {dy}) clip {clip:?}");
        }
    }

    /// `scroll(dx, dy, SCROLL_ERASE)` - Werte aus echtem pygame-ce 2.5.7.
    #[test]
    fn scroll_loeschend_wie_pygame() {
        #[allow(clippy::type_complexity)]
        let faelle: &[((i32, i32), Option<(i32, i32, i32, i32)>, [u8; 12])] = &[
            ((1, 0), None, [0, 1, 11, 21, 0, 1, 11, 21, 0, 1, 11, 21]),
            ((-1, 0), None, [11, 21, 31, 0, 11, 21, 31, 0, 11, 21, 31, 0]),
            ((0, 1), None, [0, 0, 0, 0, 1, 11, 21, 31, 1, 11, 21, 31]),
            ((2, 1), None, [0, 0, 0, 0, 0, 0, 1, 11, 0, 0, 1, 11]),
            // **Kein Modulo**: ist alles freigeworden, wird alles geloescht -
            // anders als bei `REPEAT` und anders als bei Flagge 0, die dann
            // gar nichts tut.
            ((5, 0), None, [0; 12]),
            ((-5, 0), None, [0; 12]),
            ((1, 0), Some((1, 0, 2, 3)), [1, 0, 11, 31, 1, 0, 11, 31, 1, 0, 11, 31]),
            ((0, 1), Some((1, 0, 2, 3)), [1, 0, 0, 31, 1, 11, 21, 31, 1, 11, 21, 31]),
            ((0, 1), Some((0, 1, 4, 2)), [1, 11, 21, 31, 0, 0, 0, 0, 1, 11, 21, 31]),
        ];
        for &((dx, dy), clip, soll) in faelle {
            let mut s = scroll_flaeche();
            if let Some(c) = clip {
                s.set_clip(Some(Rect::new(c.0, c.1, c.2, c.3)));
            }
            s.scroll_loeschend(dx, dy);
            assert_eq!(scroll_rot(&s), soll.to_vec(), "({dx}, {dy}) clip {clip:?}");
        }
    }

    /// Flagge 0 laesst den freigewordenen Rand **stehen** - der Unterschied
    /// zu den beiden anderen an einer Zeile.
    #[test]
    fn scroll_drei_varianten_sind_verschieden() {
        let mach = |f: fn(&mut Surface, i32, i32)| {
            let mut s = scroll_flaeche();
            f(&mut s, 1, 0);
            scroll_rot(&s)[0]
        };
        assert_eq!(mach(|s, x, y| s.scroll(x, y)), 1, "Flagge 0: Rand bleibt");
        assert_eq!(mach(|s, x, y| s.scroll_wiederholend(x, y)), 31, "REPEAT: laeuft herum");
        assert_eq!(mach(|s, x, y| s.scroll_loeschend(x, y)), 0, "ERASE: Rand wird Null");
    }

    /// Eine Zeile, gruen bei x=1, sonst rot, mit Flaechenalpha 128.
    fn flaechenalpha_zeile(breite: u32) -> Surface {
        let mut s = Surface::new((breite, 1));
        s.fill(Color::rgb(255, 0, 0));
        s.fill_rect(Color::rgb(0, 255, 0), Rect::new(1, 0, 1, 1));
        s.set_alpha(Some(128));
        s
    }

    /// Der ueberlappende Selbstblit rechnet mit pygames Formel, der
    /// nicht ueberlappende mit SDLs - und `fblits` immer mit SDLs.
    ///
    /// Die Zahlen sind an echtem pygame-ce 2.5.7 abgelesen, nicht
    /// hergeleitet: gruen (0,255,0) auf rot (255,0,0) mit Alpha 128.
    ///
    /// Abgelesen wird immer `(4, 0)` - dort landet die Quelle `(1, 0)`.
    /// Bei Breite 4 gibt es dieses Pixel nicht (das Ziel ist eine Spalte
    /// breit und liest Rot), deshalb faengt die Reihe bei 5 an.
    ///
    /// ```text
    /// Breite  5  blit  (127, 127, 0)   Bereiche beruehren sich nicht
    /// Breite  6  blit  (127, 127, 0)
    /// Breite  8  blit  (127, 128, 0)   ab hier ueberlappen sie
    /// Breite  8  fblit (127, 127, 0)   srcrect ist NULL
    /// ```
    ///
    /// Der Unterschied ist ein einziger Summand (`+ sC` in
    /// `ALPHA_BLEND_COMP`) und ein einziges Pixel - und daran hing
    /// `surface_test.test_blanket_alpha` in allen vier Tiefen.
    #[test]
    fn selbstblit_mit_flaechenalpha_nur_bei_ueberlappung_wie_pygame() {
        let gruen_bei = |s: &Surface, x: i32| s.get_at((x, 0)).g;
        for (breite, soll) in [(5u32, 127u8), (6, 127), (8, 128), (16, 128)] {
            let mut s = flaechenalpha_zeile(breite);
            let kopie = s.clone();
            s.blit_ex(&kopie, (3, 0), None, 0);
            assert_eq!(
                gruen_bei(&s, 4),
                soll,
                "Breite {breite}: gemessen an echtem pygame"
            );
        }
        // Dieselbe Lage, aber ueber `fblits`: pygame uebergibt dort kein
        // Quellrechteck, also bleibt es bei SDLs Rechnung.
        let mut s = flaechenalpha_zeile(8);
        let kopie = s.clone();
        s.fblit(&kopie, (3, 0), 0);
        assert_eq!(gruen_bei(&s, 4), 127, "fblits nimmt SDLs Weg");
    }
}
