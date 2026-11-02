//! pixelkopie.rs
//! =============
//! `pygame.pixelcopy` - Pixel zwischen Flaechen und **fremden Feldern**
//! schaufeln.
//!
//! ## Was hier einmal stand
//!
//! Vier Namen, die alle denselben `NotImplementedError` warfen, mit einer
//! ausfuehrlichen Begruendung: `pixelcopy` arbeite auf mehrdimensionalen
//! Feldern, rustgame habe keine Feld-Schnittstelle, numpy sei nicht
//! installiert, und der Plan habe das Puffer-Protokoll ausdruecklich
//! ausgeschlossen. **Drei der vier Saetze sind nicht mehr wahr.**
//!
//! Das Puffer-Protokoll gibt es ([`crate::puffer`]), damit die
//! Feld-Schnittstelle, und die Gegenseite zum Messen lag die ganze Zeit
//! daneben: `pygame.tests.test_utils.buftools.Exporter` baut ein Feld mit
//! jeder gewuenschten Form, Schrittweite und Byteordnung, und
//! `arrinter.Array` eines mit rohem `__array_struct__`. Fuer numpy braucht
//! man keins von beiden.
//!
//! Die Lehre ist dieselbe wie bei OPUS, WavPack und XM: eine Grenze, die
//! an einem fehlenden Werkzeug haengt, gehoert nachgesehen und nicht
//! abgeschrieben.
//!
//! ## Was gemessen ist
//!
//! ```text
//! make_surface(Feld 2D, itemsize 1/2/4/8)  -> Flaeche mit 8 Bit, Masken (0,0,0,0)
//! make_surface(Feld (w,h,3))               -> 32 Bit, Masken (FF0000, FF00, FF, 0)
//! make_surface(Feld (w,h,4))               -> ValueError must be a valid 2d or 3d array
//! make_surface(Feld (n,))                  -> ValueError must be a valid 2d or 3d array
//! surface_to_array(Feld (4,5), Flaeche 3x5)-> ValueError Expected a (3, 5) target: got (4, 5)
//! surface_to_array(Feld 'B', Flaeche 32Bit)-> ValueError Expected at least a target byte size of 4: got 1
//! surface_to_array(.., .., 'X')            -> TypeError unrecognized view kind 'X' for argument 3
//! ```
//!
//! Die erste Zeile ist die, auf die man nicht kommt: **jedes**
//! zweidimensionale Feld wird eine Achtbitflaeche, ganz gleich, wie breit
//! seine Eintraege sind. Ein Feld aus `Uint32` gibt keine 32-Bit-Flaeche.
//!
//! Und die Sorten von `surface_to_array`, ebenfalls gemessen:
//!
//! | `kind` | Wert je Stelle |
//! |---|---|
//! | `'P'` | der Pixelwert im Format der Flaeche |
//! | `'R'`, `'G'`, `'B'` | der Farbkanal, 0 bis 255 |
//! | `'A'` | das Alpha - **255**, wenn die Flaeche keines hat |
//! | `'C'` | `opaque`, wo kein Farbschluessel liegt, sonst `clear` |
//!
//! ## Was **nicht** geht, und warum
//!
//! Ein Feld, das auf die Pixel einer Flaeche mit 8, 16 oder 24 Bit zeigt.
//! rustgame legt jede Flaeche als ARGB8888 ab; ein Puffer darauf muesste
//! Bytes zeigen, die so nicht im Speicher stehen, und gibt darum einen
//! `BufferError`. Die Flaeche als **Quelle oder Ziel** ist davon nicht
//! betroffen - nur `Surface.get_view(...)` als Feld. Das kostet vier Tests
//! in `pixelcopy_test`; die Grenze steht in `README.md`.

use pyo3::exceptions::{PyTypeError, PyValueError};
use pyo3::prelude::*;

use crate::flaeche::PySurface;
use crate::puffer::{self, Art, Sicht};

/// Die Sicht auf ein Feld, das als Quelle oder Ziel taugt.
///
/// `hol_sicht` nimmt dabei alles, was pygame auch nimmt: einen
/// `BufferProxy`, ein `PixelArray`, ein `Color`, ein `buftools.Exporter`,
/// ein `arrinter.Array` - oder irgendetwas mit `__array_struct__`,
/// `__array_interface__` oder dem Puffer-Protokoll.
fn feld(o: &Bound<'_, PyAny>) -> PyResult<Sicht> {
    puffer::hol_sicht(o)
}

/// Der Byteversatz eines Elements.
fn versatz(s: &Sicht, idx: &[usize]) -> isize {
    idx.iter()
        .zip(s.strides.iter())
        .map(|(i, st)| *i as isize * *st)
        .sum()
}

/// Darf aus diesem Format eine ganze Zahl gelesen werden?
///
/// Gemessen an `pixelcopy_test.test_format_newbuf`, das fuenfzehn gute und
/// elf schlechte Formate durchprobiert. Gut sind `=i =I =l =L =q =Q <i >i
/// !i 1i =1i @q q 4x 8x`, schlecht `f d ? x 1x 2x 3x 5x 6x 7x 9x`.
///
/// Zwei Regeln fallen daraus: die **Sorte** muss eine ganze Zahl oder ein
/// Fuellbyte sein - `f`, `d` und `?` fallen aus -, und die **Breite** muss
/// eine sein, die eine Maschine als Zahl lesen kann: 1, 2, 4 oder 8. Genau
/// deshalb geht `4x` und `3x` nicht.
fn zahl_erlaubt(s: &Sicht, bytes: usize) -> bool {
    let f = s.format.to_string_lossy();
    // **Oder genau so breit wie ein Pixel.** Die Sicht auf eine
    // 24-Bit-Flaeche meldet `3x` mit drei Bytes, und das ist keine Zahl,
    // die eine Maschine kennt - fuer `pixelcopy` ist es trotzdem gueltig,
    // weil es genau ein Pixel fasst. Gemessen: `map_array` auf eine
    // 24-Bit-Flaeche geht, `array_to_surface` mit `3x` auf eine
    // 32-Bit-Flaeche nicht (dort greift die Breitenpruefung).
    matches!(puffer::art(&f), Art::Ganz | Art::Fuell)
        && (matches!(s.itemsize, 1 | 2 | 4 | 8) || s.itemsize == bytes)
}

/// Prueft Sorte und Breite gegen die Flaeche.
fn breite_pruefen(s: &Sicht, bytes: usize) -> PyResult<()> {
    if !zahl_erlaubt(s, bytes) {
        return Err(PyValueError::new_err(format!(
            "unsupported array item format '{}'",
            s.format.to_string_lossy()
        )));
    }
    if s.itemsize < bytes {
        return Err(PyValueError::new_err(format!(
            "Expected at least a target byte size of {}: got {}",
            bytes, s.itemsize
        )));
    }
    Ok(())
}

/// Prueft, dass die ersten zwei Achsen zur Flaeche passen.
fn form_pruefen(s: &Sicht, groesse: (u32, u32)) -> PyResult<()> {
    let (w, h) = (groesse.0 as isize, groesse.1 as isize);
    if s.shape.len() < 2 || s.shape[0] != w || s.shape[1] != h {
        let ist: Vec<String> = s.shape.iter().map(|v| v.to_string()).collect();
        return Err(PyValueError::new_err(format!(
            "Expected a ({}, {}) target: got ({})",
            w,
            h,
            ist.join(", ")
        )));
    }
    Ok(())
}

/// Die Groesse einer Flaeche, ausgeliehen.
fn groesse(s: &Bound<'_, PySurface>) -> PyResult<(u32, u32)> {
    let g = s.borrow();
    let k = g.leihen()?;
    Ok(k.get_size())
}

// ---------------------------------------------------------------------------
//  surface_to_array
// ---------------------------------------------------------------------------

/// `pygame.pixelcopy.surface_to_array(array, surface, kind='P', opaque=255, clear=0)`.
///
/// Zwei Formen, und der Unterschied steckt allein in der Achsenzahl:
/// ein Feld `(w, h)` bekommt **einen** Wert je Stelle, ein Feld `(w, h, 3)`
/// die drei Farbbytes.
#[pyfunction]
#[pyo3(signature = (array, surface, kind="P", opaque=255, clear=0))]
#[pyo3(name = "surface_to_array")]
pub fn kopie_surface_to_array(
    array: &Bound<'_, PyAny>,
    surface: &Bound<'_, PySurface>,
    kind: &str,
    opaque: i64,
    clear: i64,
) -> PyResult<()> {
    let art = kind.chars().next().unwrap_or('P');
    if !matches!(art, 'P' | 'R' | 'G' | 'B' | 'A' | 'C') || kind.chars().count() != 1 {
        return Err(PyTypeError::new_err(format!(
            "unrecognized view kind '{kind}' for argument 3"
        )));
    }
    let ziel = feld(array)?;
    if !ziel.schreibbar {
        return Err(crate::umwandlung::PygameBufferError::new_err(
            "destination buffer is read-only",
        ));
    }
    let (w, h) = groesse(surface)?;
    form_pruefen(&ziel, (w, h))?;

    let g = surface.borrow();
    let k = g.leihen()?;
    let bytes = k.get_format().bytesize() as usize;
    let schluessel = k.get_colorkey();

    match ziel.shape.len() {
        2 => {
            // Bei `'P'` muss ein Eintrag den ganzen Pixelwert fassen; bei
            // den Kanalsorten ist es immer ein Byte, und dann reicht jede
            // Breite.
            breite_pruefen(&ziel, if art == 'P' { bytes } else { 1 })?;
            for y in 0..h as i32 {
                for x in 0..w as i32 {
                    let c = k.get_at((x, y));
                    let wert = match art {
                        'P' => k.gepackt_an((x, y)) as u64,
                        'R' => c.r as u64,
                        'G' => c.g as u64,
                        'B' => c.b as u64,
                        // **Das Alpha eines Pixels zaehlt nur, solange
                        // die Flaeche mischt.** Gemessen:
                        //
                        // ```text
                        // Surface(SRCALPHA, 32), fill((10,20,30,40))
                        //   frisch               'A' -> 40
                        //   nach set_alpha(None) 'A' -> 255
                        //   nach set_alpha(0)    'A' -> 40
                        //   nach set_alpha(128)  'A' -> 40
                        // ohne SRCALPHA          'A' -> 255, immer
                        // ```
                        //
                        // `set_alpha(None)` schaltet SDLs Mischung ab, und
                        // dann ist die Flaeche undurchsichtig - auch mit
                        // Alphakanal. `surfarray_test.test_array_alpha`
                        // prueft genau diese Zeile und sagt sie im
                        // Fehlertext: "All alpha values should be 255 when
                        // surf.set_alpha(None) has been set."
                        'A' => {
                            if k.has_alpha() && k.get_blendmode() != 0 {
                                c.a as u64
                            } else {
                                255
                            }
                        }
                        _ => {
                            let ist_schluessel = schluessel
                                .map(|s| c == s)
                                .unwrap_or(false);
                            (if ist_schluessel { clear } else { opaque }) as u64
                        }
                    };
                    puffer::feld_schreiben(
                        &ziel,
                        versatz(&ziel, &[x as usize, y as usize]),
                        wert,
                    );
                }
            }
        }
        3 if ziel.shape[2] == 3 => {
            breite_pruefen(&ziel, 1)?;
            for y in 0..h as i32 {
                for x in 0..w as i32 {
                    let c = k.get_at((x, y));
                    for (n, v) in [c.r, c.g, c.b].into_iter().enumerate() {
                        puffer::feld_schreiben(
                            &ziel,
                            versatz(&ziel, &[x as usize, y as usize, n]),
                            v as u64,
                        );
                    }
                }
            }
        }
        n => {
            return Err(PyValueError::new_err(format!(
                "unsupported array depth {n}"
            )))
        }
    }
    Ok(())
}

// ---------------------------------------------------------------------------
//  array_to_surface
// ---------------------------------------------------------------------------

/// `pygame.pixelcopy.array_to_surface(surface, array)`.
///
/// Der Rueckweg, und er kann **ausbreiten**: ein Feld `(1, h)` fuellt jede
/// Spalte, ein Feld `(w, 1)` jede Zeile, ein Feld `(1, 1)` die ganze
/// Flaeche. Gemessen an `test_array_to_surface_broadcasting`, das alle drei
/// Formen gegen jede Flaechentiefe durchprobiert.
#[pyfunction]
#[pyo3(name = "array_to_surface")]
#[pyo3(signature = (surface, array, /))]
pub fn kopie_array_to_surface(
    surface: &Bound<'_, PySurface>,
    array: &Bound<'_, PyAny>,
) -> PyResult<()> {
    let quelle = feld(array)?;
    let (w, h) = groesse(surface)?;
    let g = surface.borrow();
    let mut k = g.leihen_mut()?;
    let bytes = k.get_format().bytesize() as usize;

    // Welche Achse traegt wirklich Werte, und welche wird ausgebreitet?
    let passt = |ist: isize, soll: u32| ist == soll as isize || ist == 1;
    match quelle.shape.len() {
        2 if passt(quelle.shape[0], w) && passt(quelle.shape[1], h) => {
            breite_pruefen(&quelle, bytes)?;
            for y in 0..h as i32 {
                let qy = if quelle.shape[1] == 1 { 0 } else { y as usize };
                for x in 0..w as i32 {
                    let qx = if quelle.shape[0] == 1 { 0 } else { x as usize };
                    // **Der Wert ist schon der Pixelwert.** Er kommt aus
                    // einem Feld, das pygames `map_rgb`-Zahlen traegt, und
                    // geht unveraendert in den Speicher - nicht ueber eine
                    // Farbe, die auf einer Palettenflaeche einen anderen
                    // Index zurueckgeben wuerde.
                    let v = puffer::feld_lesen(&quelle, versatz(&quelle, &[qx, qy])) as u32;
                    k.setz_gepackt((x, y), v);
                }
            }
        }
        3 if quelle.shape[2] == 3
            && passt(quelle.shape[0], w)
            && passt(quelle.shape[1], h) =>
        {
            // **Ein dreiachsiges Feld braucht eine Flaeche mit Farbkanaelen.**
            //
            // Auf einer Palettenflaeche gibt es keine: jedes Pixel ist ein
            // Index, und drei Bytes hineinzurechnen hiesse, sie auf die
            // Tafel zu runden - eine stille Naeherung statt einer Kopie.
            // pygame lehnt ab, und `surfarray_test.test_blit_array` prueft
            // es: `if bitsize == 8: assertRaises(ValueError, do_blit, ...)`.
            if k.get_palette().is_some() {
                return Err(PyValueError::new_err(
                    "unsupported colormasks for 3D reference array",
                ));
            }
            breite_pruefen(&quelle, 1)?;
            for y in 0..h as i32 {
                let qy = if quelle.shape[1] == 1 { 0 } else { y as usize };
                for x in 0..w as i32 {
                    let qx = if quelle.shape[0] == 1 { 0 } else { x as usize };
                    let mut c = [0u8; 3];
                    for (n, ziel) in c.iter_mut().enumerate() {
                        *ziel = puffer::feld_lesen(&quelle, versatz(&quelle, &[qx, qy, n]))
                            as u8;
                    }
                    let farbe = rustgame_core::color::Color::rgb(c[0], c[1], c[2]);
                    let wort = k.abbilden(farbe);
                    k.set_at_ungeprueft((x, y), wort);
                }
            }
        }
        2 | 3 => {
            let ist: Vec<String> = quelle.shape.iter().map(|v| v.to_string()).collect();
            return Err(PyValueError::new_err(format!(
                "Expected a ({}, {}) array: got ({})",
                w,
                h,
                ist.join(", ")
            )));
        }
        n => {
            return Err(PyValueError::new_err(format!(
                "unsupported array depth {n}"
            )))
        }
    }
    Ok(())
}

// ---------------------------------------------------------------------------
//  map_array
// ---------------------------------------------------------------------------

/// `pygame.pixelcopy.map_array(array, array, surface)`.
///
/// Rechnet Farbtripel in Pixelwerte um - die Flaeche ist dabei nur das
/// **Format**, kein Ziel. Die Quelle darf auch eine einzelne Farbe sein:
/// `map_array(t.get_view("2"), Color("salmon"), t)` faerbt die ganze
/// Flaeche, und das geht, weil `Color` seit kurzem selbst einen Puffer der
/// Laenge drei herausgibt.
#[pyfunction]
#[pyo3(name = "map_array")]
#[pyo3(signature = (array1, array2, surface, /))]
pub fn kopie_map_array(
    array1: &Bound<'_, PyAny>,
    array2: &Bound<'_, PyAny>,
    surface: &Bound<'_, PySurface>,
) -> PyResult<()> {
    // Die oeffentlichen Namen sind pygames (`pixelcopy.pyi`); die
    // deutschen leben im Rumpf weiter.
    let (ziel, quelle) = (array1, array2);
    let z = feld(ziel)?;
    let q = feld(quelle)?;
    if !z.schreibbar {
        return Err(crate::umwandlung::PygameBufferError::new_err(
            "destination buffer is read-only",
        ));
    }
    if z.shape.len() != 2 {
        return Err(PyValueError::new_err(format!(
            "unsupported array depth {}",
            z.shape.len()
        )));
    }
    let (w, h) = (z.shape[0] as usize, z.shape[1] as usize);
    let g = surface.borrow();
    let k = g.leihen()?;
    breite_pruefen(&z, k.get_format().bytesize() as usize)?;

    // **Vier Formen der Quelle**, und die dritte ist die, auf die man
    // nicht kommt. Gemessen an `pixelcopy_test.test_map_array`:
    //
    // | `shape` der Quelle | was es bedeutet |
    // |---|---|
    // | `(3,)` | **eine** Farbe, ueber alles ausgebreitet |
    // | `(n, 3)` | n Farben **uebereinander** - wie `(1, n, 3)` |
    // | `(w, 1, 3)` | eine Farbe je Spalte, ueber die Zeilen ausgebreitet |
    // | `(w, h, 3)` | je Stelle eine Farbe |
    //
    // Die zweite Zeile ist die Falle: ein zweiachsiges Feld `(3, 3)` ist
    // **kein** Feld von Pixelwerten, sondern eine Liste von drei Farben.
    // Der Test legt es gegen ein Ziel `(4, 3)` und erwartet drei
    // Streifen - also wird die erste Achse zu `y` und nicht zu `x`.
    let achsen = q.shape.len();
    let (qw, qh) = match (achsen, &q.shape[..]) {
        (1, &[3]) => (1usize, 1usize),
        (2, &[n, 3]) => (1usize, n as usize),
        (3, s) if s[2] == 3 => (s[0] as usize, s[1] as usize),
        _ => {
            return Err(PyValueError::new_err(
                "expected a (w, h, 3) source array or a single color",
            ))
        }
    };
    if achsen > 1 && ((qw != w && qw != 1) || (qh != h && qh != 1)) {
        return Err(PyValueError::new_err(format!(
            "Source and destination array shapes do not match: ({qw}, {qh}) != ({w}, {h})"
        )));
    }
    for y in 0..h {
        for x in 0..w {
            let mut c = [0u8; 3];
            for (n, ziel) in c.iter_mut().enumerate() {
                let v = match achsen {
                    1 => versatz(&q, &[n]),
                    // `(n, 3)`: die erste Achse ist `y`, die zweite der Kanal.
                    2 => versatz(&q, &[if qh == 1 { 0 } else { y }, n]),
                    _ => versatz(
                        &q,
                        &[if qw == 1 { 0 } else { x }, if qh == 1 { 0 } else { y }, n],
                    ),
                };
                *ziel = puffer::feld_lesen(&q, v) as u8;
            }
            let farbe = rustgame_core::color::Color::rgb(c[0], c[1], c[2]);
            puffer::feld_schreiben(
                &z,
                versatz(&z, &[x, y]),
                k.map_rgb_gepackt(farbe) as u64,
            );
        }
    }
    Ok(())
}

// ---------------------------------------------------------------------------
//  make_surface
// ---------------------------------------------------------------------------

/// `pygame.pixelcopy.make_surface(array)`.
///
/// **Ein zweidimensionales Feld gibt eine Flaeche mit acht Bit**, ganz
/// gleich, wie breit seine Eintraege sind - gemessen fuer `B`, `=H`, `=I`
/// und `=Q`, und jedes Mal kam `get_bitsize() == 8` mit den Masken
/// `(0, 0, 0, 0)` heraus. Wer aus einem `Uint32`-Feld eine 32-Bit-Flaeche
/// erwartet, irrt.
///
/// Ein dreidimensionales Feld `(w, h, 3)` gibt 32 Bit mit den Masken
/// `(0xFF0000, 0xFF00, 0xFF, 0)`. Jede andere dritte Achse ist ein
/// `ValueError`, und eine Achse allein auch.
#[pyfunction]
#[pyo3(name = "make_surface")]
#[pyo3(signature = (array, /))]
pub fn kopie_make_surface(array: &Bound<'_, PyAny>) -> PyResult<Py<PyAny>> {
    let py = array.py();
    let s = feld(array)?;
    let (w, h, tiefe) = match s.shape.len() {
        2 => (s.shape[0], s.shape[1], 8u32),
        3 if s.shape[2] == 3 => (s.shape[0], s.shape[1], 32),
        _ => return Err(PyValueError::new_err("must be a valid 2d or 3d array")),
    };
    use rustgame_core::surface::{palette_vorgabe, Format, Surface as KernSurface};
    let mut k = KernSurface::new((w as u32, h as u32));
    k.set_format(Format::standard(tiefe, false).map_err(PyValueError::new_err)?);
    if tiefe == 8 {
        k.set_palette(Some(palette_vorgabe()));
    }
    let gebunden = Bound::new(py, PySurface::neu(k))?;
    kopie_array_to_surface(&gebunden, array)?;
    Ok(gebunden.into_any().unbind())
}
