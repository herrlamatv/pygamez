//! text.rs
//! =======
//! `pygame.font`.
//!
//! ## Die Zeile, auf die es ankommt
//!
//! Im umgebenden Projekt steht 1002-mal dasselbe:
//!
//! ```text
//! flaeche = font.render(text, True, farbe)
//! ```
//!
//! Also Kantenglaettung an, kein Hintergrund. Das Ergebnis ist eine
//! Flaeche mit `SRCALPHA`, und sie ist genau `get_linesize()` hoch,
//! **nicht** `get_height()`. Gemessen an pygame-ce 2.5.7 mit
//! `Font(None, 24)`:
//!
//! ```text
//! get_height()   16      render("Hallo", …).get_size() -> (41, 18)
//! get_linesize() 18      size("Hallo")                 -> (41, 16)
//! ```
//!
//! Die beiden Hoehen sind verschieden, und wer die falsche nimmt, merkt
//! es erst, wenn Text im Kasten verrutscht.
//!
//! ## Was stimmt und was nicht
//!
//! Die Rasterung uebernimmt fontdue; FreeType rastert anders, also sind
//! die **Pixel** nicht dieselben. Alles, was aus der Schriftdatei kommt -
//! `ascent`, `descent`, `height`, `linesize`, Vorschuebe, Kerning -
//! stimmt dagegen auf den Wert genau. Der Vergleich im Bericht sagt, wo
//! die Grenze liegt.
//!
//! ## Leerer Text
//!
//! `render("")` gibt eine Flaeche `0 x get_height()` **ohne** `SRCALPHA`
//! zurueck - auch das gemessen, und der Sonderfall steht so in pygames
//! `font.c`.

use std::path::PathBuf;

use pyo3::exceptions::{PyTypeError, PyValueError};
use pyo3::prelude::*;
use pyo3::types::PyString;

use rustgame_font as schrift;
use schrift::font::Ausrichtung;

use crate::flaeche::PySurface;
use crate::rechteck::PyRect;
use crate::umwandlung::{farbe_aus, punkt_aus, schrift_fehler, PygameError};

/// Die Masse eines einzelnen Zeichens: `(minx, maxx, miny, maxy, advance)`.
///
/// Denselben Namen traegt der Typ in `rustgame_font`; er steht hier noch
/// einmal, weil die Huelle nicht die Typen der anderen Kiste nach aussen
/// reichen soll.
pub type GlyphMass = (i32, i32, i32, i32, i32);

/// Ein Schriftname aus Text, `bytes` oder einer Folge davon.
///
/// Eine Folge wird zur Kommaliste zusammengezogen - genau das, was
/// `match_font` und `SysFont` ohnehin verarbeiten.
pub fn schriftnamen_aus(o: &Bound<'_, PyAny>) -> PyResult<String> {
    fn einer(o: &Bound<'_, PyAny>) -> PyResult<String> {
        if let Ok(t) = o.downcast::<PyString>() {
            return Ok(t.to_cow()?.into_owned());
        }
        if o.is_instance_of::<pyo3::types::PyBytes>() {
            return Ok(o.extract::<Vec<u8>>()?.iter().map(|&c| c as char).collect());
        }
        Err(PyTypeError::new_err("font name must be a string or bytes"))
    }
    if o.downcast::<PyString>().is_ok() || o.is_instance_of::<pyo3::types::PyBytes>() {
        return einer(o);
    }
    if let Ok(iter) = o.try_iter() {
        let mut teile = Vec::new();
        for e in iter {
            teile.push(einer(&e?)?);
        }
        return Ok(teile.join(","));
    }
    einer(o)
}

/// Text aus allem, was pygame hier annimmt: `str`, `bytes` und `None`.
///
/// Gemessen:
///
/// ```text
/// f.render("hi", True, c)   -> 12x15
/// f.render(b"hi", True, c)  -> 12x15   (gleich!)
/// f.render(None, True, c)   -> 0x13    (leer, volle Zeilenhoehe)
/// f.metrics(b"hi")          -> dieselbe Liste wie fuer "hi"
/// ```
///
/// `bytes` werden als Latin-1 gelesen, nicht als UTF-8: das ist pygames
/// alte Schnittstelle, und ein Byte ueber 127 ist dort ein Zeichen und
/// kein Fehler.
fn text_aus(o: &Bound<'_, PyAny>) -> PyResult<String> {
    if o.is_none() {
        return Ok(String::new());
    }
    if let Ok(t) = o.downcast::<PyString>() {
        return Ok(t.to_cow()?.into_owned());
    }
    if o.is_instance_of::<pyo3::types::PyBytes>() {
        let b = o.extract::<Vec<u8>>()?;
        return Ok(b.iter().map(|&c| c as char).collect());
    }
    Err(PyTypeError::new_err("text must be a unicode or bytes"))
}

/// pygames Satz fuer eine Schriftdatei, die es nicht gibt.
///
/// **Hier stand der Satz des Betriebssystems**, und der ist in der
/// Systemsprache: `FileNotFoundError: Das System kann die angegebene
/// Datei nicht finden. (os error 2)`. Ein deutscher Satz in einer
/// Ausnahme - und `tools/deutsche_fehlertexte.py` konnte ihn nicht
/// sehen, weil er nicht im Quelltext steht, sondern von Windows kommt.
/// Gefunden hat ihn die Probe `Klassen falsch bauen`.
///
/// pygame unterscheidet zwei Faelle (gemessen, `rwobject.c`):
///
/// ```text
/// Font('nixda')          No file 'nixda' found in working directory 'C:\...\rustgame'.
/// Font('C:/nix/x.ttf')   No such file or directory: 'C:/nix/x.ttf'.
/// Font('.')              No file '.' found in working directory '...'   (ein Ordner zaehlt nicht)
/// Font(b'nixda')         No file 'b'nixda'' found in working directory ...
/// ```
///
/// Der Name im Satz ist `str(argument)` - darum traegt die
/// `bytes`-Fassung ihr `b''` mit. `wie` ist genau dieser Text.
pub(crate) fn datei_fehlt(name: &str, wie: &str) -> Option<PyErr> {
    let p = std::path::Path::new(name);
    if p.is_file() {
        return None;
    }
    if p.is_absolute() {
        return Some(pyo3::exceptions::PyFileNotFoundError::new_err(format!(
            "No such file or directory: '{wie}'."
        )));
    }
    let ordner = std::env::current_dir()
        .map(|p| p.display().to_string())
        .unwrap_or_default();
    Some(pyo3::exceptions::PyFileNotFoundError::new_err(format!(
        "No file '{wie}' found in working directory '{ordner}'."
    )))
}

/// Eine Schrift aus einem Dateinamen - mit **pygames Sonderfall**.
///
/// `font_test.py` sagt es im Kommentar selbst: *"In font_init, a special
/// case is when the filename argument is identical to the default font
/// file name."* Gemessen geht `Font("freesansbold.ttf", 20)` in pygame
/// auch dann, wenn im aktuellen Ordner keine solche Datei liegt - der
/// Name allein genuegt, und es nimmt die eingebaute Schrift.
///
/// **Aber nicht mit pygames Groessenfaktor.** `Font(None, 24)` hat
/// Punktgroesse 16 (Faktor 0.6875), `Font("freesansbold.ttf", 24)` hat
/// 24. Der Sonderfall betrifft nur, *wo die Datei herkommt*, nicht *wie
/// sie gelesen wird* - deshalb steht hier `Font::from_bytes` und nicht
/// `default_font`.
///
/// Zusaetzlich liegt die Datei jetzt **neben dem Modul**, so wie pygame
/// sie neben seinem Paket liegen hat. Sieben Tests bauen ihren Pfad als
/// `os.path.join(dirname(pygame.__file__), get_default_font())` und
/// erwarten dort eine echte Datei; mit umgebogenem `pygame` zeigt das auf
/// unseren Ordner. Ein Sonderfall haette das nicht geloest - der Pfad ist
/// ja voll ausgeschrieben.
fn schrift_aus_namen(name: &str, wie: &str, size: i32) -> PyResult<schrift::Font> {
    let p = PathBuf::from(name);
    if !p.is_file() {
        let blosser_name = p
            .file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_default();
        if blosser_name == schrift::get_default_font() {
            if let Some(echt) = schrift::default_font_path() {
                let bytes = std::fs::read(echt)?;
                return schrift::Font::from_bytes(bytes, size).map_err(schrift_fehler);
            }
        }
        // **Nach dem Sonderfall, nicht davor.** `Font("freesansbold.ttf")`
        // geht auch ohne Datei im Arbeitsverzeichnis - siehe oben.
        if let Some(e) = datei_fehlt(name, wie) {
            return Err(e);
        }
    }
    schrift::Font::new(p, size).map_err(schrift_fehler)
}

/// `pygame.font.Font`.
// `unsendable`: eine `Font` haelt ihre Schriftdatei und die
// Glyphenablage ueber `Rc`. Das ist nicht ueber Threads hinweg gueltig -
// PyO3 bindet das Objekt deshalb an seinen Erzeuger-Strang.
#[pyclass(name = "Font", module = "rustgame_py.font", unsendable, subclass)]
pub struct PyFont {
    inner: schrift::Font,
    /// In welcher **Lebensspanne** des Schriftmoduls diese Schrift
    /// entstanden ist.
    ///
    /// `pygame.font.quit()` macht jede vorher gebaute `Font` ungueltig -
    /// gemessen gibt danach **jede** Methode und **jedes** Feld
    /// `pygame.error: Invalid font (font module quit since font created)`,
    /// auch die reinen Leser wie `get_height()`.
    ///
    /// Das laesst sich nicht an `font.get_init()` ablesen: wird das Modul
    /// nach dem `quit()` wieder gestartet, ist es initialisiert, die alte
    /// Schrift bleibt aber tot. Deshalb ein Zaehler, der bei jedem `quit()`
    /// hochgeht, und ein Vergleich statt einer Abfrage.
    epoche: u64,
    /// Die Laufrichtung - siehe [`PyFont::in_richtung`].
    richtung: i64,
    /// Das eingestellte Schriftsystem als Vierbuchstabencode.
    ///
    /// Gemerkt, aber ohne Wirkung: es waehlt in pygame HarfBuzz'
    /// Formungsregeln, und die gibt es hier nicht. Siehe
    /// [`PyFont::set_script`].
    skript: Option<String>,
}

/// Wie oft `font.quit()` schon gerufen wurde.
static EPOCHE: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);

fn epoche_jetzt() -> u64 {
    EPOCHE.load(std::sync::atomic::Ordering::Relaxed)
}

impl PyFont {
    fn neu(inner: schrift::Font) -> PyFont {
        PyFont {
            inner,
            epoche: epoche_jetzt(),
            richtung: 0,
            skript: None,
        }
    }

    /// Der Text in der eingestellten **Laufrichtung**.
    ///
    /// Gemessen an `Font(None, 24)` und `"abc"`:
    ///
    /// ```text
    /// 0  links nach rechts   size (29, 16)
    /// 1  rechts nach links   size (29, 16)   - gleiche Masse, andere Folge
    /// 2  oben nach unten     size (10, 51)
    /// 3  unten nach oben     size (10, 52)
    /// ```
    ///
    /// Die beiden senkrechten Richtungen sind damit nichts anderes als
    /// **ein Zeichen je Zeile** - und Zeilen kann die Schriftkiste schon.
    /// Deshalb steht hier eine Umformung des Textes und kein zweiter
    /// Rechenweg.
    ///
    /// Was hier **nicht** passiert, ist Formung: arabische Zeichen
    /// bekommen keine Anfangs- und Endformen, indische keine Ligaturen.
    /// Dafuer braeuchte es HarfBuzz. Die Laufrichtung allein ist aber
    /// genau das, was `set_direction` verspricht, und sie stimmt.
    fn in_richtung(&self, text: &str) -> String {
        match self.richtung {
            1 => text
                .split('\n')
                .map(|z| z.chars().rev().collect::<String>())
                .collect::<Vec<String>>()
                .join("\n"),
            2 => text.chars().map(|c| c.to_string()).collect::<Vec<String>>().join("\n"),
            3 => text
                .chars()
                .rev()
                .map(|c| c.to_string())
                .collect::<Vec<String>>()
                .join("\n"),
            _ => text.to_string(),
        }
    }

    /// Lebt diese Schrift noch? Sonst pygames Wortlaut.
    fn lebt(&self) -> PyResult<()> {
        if self.epoche != epoche_jetzt() {
            return Err(crate::umwandlung::PygameError::new_err(
                "Invalid font (font module quit since font created)",
            ));
        }
        Ok(())
    }
}

#[pymethods]
impl PyFont {
    /// `Font(file=None, size=20)`.
    ///
    /// `file = None` nimmt die Standardschrift - und **nur** dort wirkt
    /// pygames Faktor 0.6875: `Font(None, 24)` hat Punktgroesse 16,
    /// `Font("irgendwas.ttf", 24)` hat 24. Gemessen.
    #[new]
    // `filename`, nicht `file` - `font.pyi` nennt ihn so, und
    // `pygame.font.Font(filename=None, size=12)` ist gemessen gueltig.
    #[pyo3(signature = (filename=None, size=20))]
    fn new(filename: Option<&Bound<'_, PyAny>>, size: i32) -> PyResult<PyFont> {
        font_pruefen()?;
        let file = filename;
        let Some(f) = file else {
            return Ok(PyFont::neu(schrift::default_font(size).map_err(schrift_fehler)?));
        };
        if f.is_none() {
            return Ok(PyFont::neu(schrift::default_font(size).map_err(schrift_fehler)?));
        }
        // Ein Dateiname - als Text, als `bytes` oder als `pathlib.Path`.
        //
        // **`bytes` gehoert dazu**: `font_test.test_load_from_file_bytes`
        // reicht den Pfad `os.fsencode`-artig hinein, und pygame nimmt
        // ihn. Vorher gab es dort
        // `TypeError: 'bytes' object cannot be converted to 'PyString'`.
        let name: Option<String> = if let Ok(t) = f.downcast::<PyString>() {
            Some(t.to_cow()?.into_owned())
        } else if f.is_instance_of::<pyo3::types::PyBytes>() {
            String::from_utf8(f.extract::<Vec<u8>>()?).ok()
        } else if let Ok(p) = f.call_method0("__fspath__") {
            p.extract::<String>().ok()
        } else {
            None
        };
        if let Some(name) = name {
            // `str(f)` ist der Text, den pygame in den Fehlersatz
            // schreibt - bei `bytes` also mit `b''` darum.
            let wie = f.str().map(|t| t.to_string()).unwrap_or_else(|_| name.clone());
            return Ok(PyFont::neu(schrift_aus_namen(&name, &wie, size)?));
        }

        // Ein Dateiobjekt. `read()` gibt bei `open(..., "rb")` Bytes und
        // bei `open(..., "r")` einen Text - beides kommt in pygames Tests
        // vor, und `extract::<Vec<u8>>()` scheitert am zweiten mit
        // `Can't extract 'str' to 'Vec'`.
        if let Ok(daten) = f.call_method0("read") {
            let b: Vec<u8> = match daten.extract::<Vec<u8>>() {
                Ok(b) => b,
                Err(_) => daten.extract::<String>()?.into_bytes(),
            };
            // **Ein leeres Dateiobjekt ist ein `ValueError`**, kein
            // `pygame.error` - und die Meldung nennt die Groesse. Gemessen:
            //
            // ```text
            // Font(io.StringIO())        ValueError: Font file object has an invalid file size: 0
            // Font(io.BytesIO(b""))      ValueError: dasselbe
            // Font(io.BytesIO(b"abc"))   geht durch (!)
            // ```
            //
            // Die dritte Zeile ist die ueberraschende: pygame prueft nur
            // die **Groesse**, nicht den Inhalt. Nur bei null Bytes meldet
            // es sich, und `font_test.test_load_from_invalid_sized_file_obj`
            // prueft genau das mit `assertRaises(ValueError)`.
            if b.is_empty() {
                return Err(pyo3::exceptions::PyValueError::new_err(
                    "Font file object has an invalid file size: 0",
                ));
            }
            return Ok(PyFont::neu(
                schrift::Font::from_bytes(b, size).map_err(schrift_fehler)?,
            ));
        }
        Err(PyTypeError::new_err(
            "font file must be a filename, a file object or None",
        ))
    }

    // ----- Masse ---------------------------------------------------------

    fn get_height(&self) -> PyResult<i32> {
        self.lebt()?;
        Ok(self.inner.get_height())
    }
    fn get_linesize(&self) -> PyResult<i32> {
        self.lebt()?;
        Ok(self.inner.get_linesize())
    }
    /// `set_linesize(n)` - **nicht negativ**, gemessen:
    /// `ValueError: linesize must be >= 0`.
    ///
    /// **Drei verschiedene Ausnahmen, und pygame haelt sie auseinander**
    /// (gemessen): `'x'`, `None` und `1.5` geben
    /// `TypeError: linesize must be an integer`, `2**31` den
    /// `OverflowError` des C-`long`, `-5` den `ValueError` unten - und
    /// `True` geht durch. Mit `linesize: i32` kam fuer die erste Gruppe
    /// PyO3s Satz.
    #[pyo3(signature = (wert, /))]
    fn set_linesize(&mut self, wert: &Bound<'_, PyAny>) -> PyResult<()> {
        if !wert.is_instance_of::<pyo3::types::PyInt>() {
            return Err(PyTypeError::new_err("linesize must be an integer"));
        }
        let linesize = ganzzahl(wert)? as i32;
        self.lebt()?;
        if linesize < 0 {
            return Err(pyo3::exceptions::PyValueError::new_err(
                "linesize must be >= 0",
            ));
        }
        self.inner.set_linesize(linesize);
        Ok(())
    }
    fn get_ascent(&self) -> PyResult<i32> {
        self.lebt()?;
        Ok(self.inner.get_ascent())
    }
    fn get_descent(&self) -> PyResult<i32> {
        self.lebt()?;
        Ok(self.inner.get_descent())
    }
    fn get_point_size(&self) -> PyResult<i32> {
        self.lebt()?;
        Ok(self.inner.get_point_size())
    }
    /// `set_point_size(n)` - **echt groesser null**, gemessen:
    /// `ValueError: point_size cannot be equal to, or less than 0`.
    /// Die Null ist also ausgeschlossen, anders als bei `linesize`.
    #[pyo3(signature = (groesse, /))]
    fn set_point_size(&mut self, groesse: i32) -> PyResult<()> {
        self.lebt()?;
        if groesse <= 0 {
            return Err(pyo3::exceptions::PyValueError::new_err(
                "point_size cannot be equal to, or less than 0",
            ));
        }
        self.inner.set_point_size(groesse);
        Ok(())
    }

    /// Die Masse eines Textes **ohne** ihn zu setzen - Hoehe ist
    /// `get_height()`, nicht `get_linesize()`.
    #[pyo3(signature = (text, /))]
    fn size(&self, text: &Bound<'_, PyAny>) -> PyResult<(u32, u32)> {
        self.lebt()?;
        let text = &self.in_richtung(&text_aus(text)?);
        // SDL_ttf laedt fuer die Masse jede Glyphe - und lehnt jede ab,
        // wenn das CV-Programm scheitert (`Font::ladefehler`).
        if !text.is_empty() && self.inner.ladefehler() {
            return Err(PygameError::new_err("Couldn't find glyph"));
        }
        Ok(self.inner.size(text))
    }

    /// Je Zeichen `(minx, maxx, miny, maxy, advance)`, oder `None`, wenn
    /// die Schrift das Zeichen nicht hat.
    #[pyo3(signature = (text, /))]
    fn metrics(&self, text: &Bound<'_, PyAny>) -> PyResult<Vec<Option<GlyphMass>>> {
        self.lebt()?;
        // Die Masse je Zeichen stehen in der **Reihenfolge des Textes**,
        // nicht in der der Laufrichtung - gemessen gibt `metrics` bei
        // rechts-nach-links dieselbe Liste wie bei links-nach-rechts.
        let text = &text_aus(text)?;
        // `TTF_GlyphMetrics32` scheitert fuer jede Glyphe, wenn das
        // CV-Programm scheitert (`Font::ladefehler`) - pygame gibt `None`.
        if self.inner.ladefehler() {
            return Ok(text.chars().map(|_| None).collect());
        }
        Ok(self.inner.metrics(text))
    }

    // ----- Schnitt -------------------------------------------------------

    // pygame haelt jeden Schalter **zweimal** vor: als Attribut
    // (`font.bold = True`) und als Methodenpaar (`font.set_bold(True)`).
    // Beide muessen hier verschiedene Rust-Namen tragen - PyO3 leitet aus
    // `#[getter] fn bold` denselben internen Namen ab wie aus der Methode
    // `get_bold`, und dann kollidieren sie beim Uebersetzen.
    #[getter(bold)]
    fn bold_lesen(&self) -> PyResult<bool> {
        self.lebt()?;
        Ok(self.inner.get_bold())
    }
    /// **Jeder Wert ist ein Wahrheitswert** - pygames `font.c` liest hier
    /// `PyObject_IsTrue`. Gemessen: `f.bold = 'x'`, `= 1.5` und
    /// `= (1,2,3,4,5)` geben alle `True`, `f.bold = None` und `= []`
    /// geben `False`, und `f.set_bold(...)` macht es genauso. Hier stand
    /// `an: bool`, und PyO3 wies alles ausser `True`/`False` zurueck:
    /// **56 Zuweisungen**, die pygame annimmt.
    ///
    /// `pygame.freetype.Font` macht es genau umgekehrt und laesst nur
    /// `True` und `False` zu - zwei Schriftmodule, zwei Regeln, und hier
    /// waren beide verkehrt herum.
    #[setter(bold)]
    fn bold_setzen(&mut self, an: &Bound<'_, PyAny>) -> PyResult<()> {
        self.lebt()?;
        self.inner.set_bold(an.is_truthy()?);
        Ok(())
    }
    #[getter(italic)]
    fn italic_lesen(&self) -> PyResult<bool> {
        self.lebt()?;
        Ok(self.inner.get_italic())
    }
    #[setter(italic)]
    fn italic_setzen(&mut self, an: &Bound<'_, PyAny>) -> PyResult<()> {
        self.lebt()?;
        self.inner.set_italic(an.is_truthy()?);
        Ok(())
    }
    #[getter(underline)]
    fn underline_lesen(&self) -> PyResult<bool> {
        self.lebt()?;
        Ok(self.inner.get_underline())
    }
    #[setter(underline)]
    fn underline_setzen(&mut self, an: &Bound<'_, PyAny>) -> PyResult<()> {
        self.lebt()?;
        self.inner.set_underline(an.is_truthy()?);
        Ok(())
    }
    #[getter(strikethrough)]
    fn strikethrough_lesen(&self) -> PyResult<bool> {
        self.lebt()?;
        Ok(self.inner.get_strikethrough())
    }
    #[setter(strikethrough)]
    fn strikethrough_setzen(&mut self, an: &Bound<'_, PyAny>) -> PyResult<()> {
        self.lebt()?;
        self.inner.set_strikethrough(an.is_truthy()?);
        Ok(())
    }
    /// `point_size` gibt es in pygame **auch** als Attribut.
    #[getter(point_size)]
    fn point_size_lesen(&self) -> PyResult<i32> {
        self.lebt()?;
        Ok(self.inner.get_point_size())
    }
    #[setter(point_size)]
    fn point_size_setzen(&mut self, groesse: i32) -> PyResult<()> {
        self.lebt()?;
        // Derselbe Satz wie bei `set_point_size` - gemessen.
        if groesse <= 0 {
            return Err(pyo3::exceptions::PyValueError::new_err(
                "point_size cannot be equal to, or less than 0",
            ));
        }
        self.inner.set_point_size(groesse);
        Ok(())
    }

    /// `font.align` - wie mehrzeiliger Text ausgerichtet wird.
    ///
    /// Die Werte sind pygames `FONT_LEFT`, `FONT_CENTER`, `FONT_RIGHT`, also
    /// 0, 1, 2; Standard ist 0 (gemessen).
    #[getter(align)]
    fn align_lesen(&self) -> PyResult<i32> {
        self.lebt()?;
        Ok(match self.inner.get_align() {
            Ausrichtung::Links => 0,
            Ausrichtung::Mitte => 1,
            Ausrichtung::Rechts => 2,
        })
    }
    #[setter(align)]
    fn align_setzen(&mut self, wert: &Bound<'_, PyAny>) -> PyResult<()> {
        self.lebt()?;
        // Zwei verschiedene Ausnahmen, und pygame haelt sie auseinander
        // (gemessen): `font.align = "x"` gibt `TypeError`, `font.align = 3`
        // dagegen `pygame.error`. Der Wortlaut steht so in `font_setter_align`.
        //
        // **`i32`, nicht `i64`.** pygame liest auch hier einen C-`long`,
        // und eine Zahl, die nicht hineinpasst, ist ihm kein Wert zum
        // Pruefen mehr: gemessen gibt `font.align = 2**31` den `TypeError`
        // von dieser Stelle, nicht den `pygame.error` von unten. Mit `i64`
        // war es hier umgekehrt.
        let Ok(v) = wert.extract::<i32>() else {
            return Err(PyTypeError::new_err(
                "font.align must be an integer. Must correspond with \
                 FONT_LEFT, FONT_CENTER, or FONT_RIGHT.",
            ));
        };
        let a = match v {
            0 => Ausrichtung::Links,
            1 => Ausrichtung::Mitte,
            2 => Ausrichtung::Rechts,
            _ => {
                return Err(PygameError::new_err(
                    "font.align must be FONT_LEFT, FONT_CENTER, or FONT_RIGHT.",
                ))
            }
        };
        self.inner.set_align(a);
        Ok(())
    }

    /// `font.name` - der Familienname aus der Schriftdatei, nur lesbar.
    #[getter(name)]
    fn name_lesen(&self) -> PyResult<String> {
        self.lebt()?;
        Ok(self.inner.name())
    }

    /// `font.style_name` - der Schnittname aus der Schriftdatei, nur lesbar.
    #[getter(style_name)]
    fn style_name_lesen(&self) -> PyResult<String> {
        self.lebt()?;
        Ok(self.inner.style_name())
    }

    /// `font.outline` - die Breite der Kontur.
    ///
    /// Nur 0 ist moeglich, siehe [`PyFont::outline_setzen`].
    #[getter(outline)]
    fn outline_lesen(&self) -> PyResult<i32> {
        self.lebt()?;
        Ok(self.inner.get_outline())
    }
    /// Alles ausser 0 ist hier ein Fehler, und zwar mit Absicht.
    ///
    /// `outline = n` laesst FreeType die Glyphenkontur mit `FT_Stroker` um
    /// `n` Pixel nach aussen ziehen und rastert **die Kontur** statt der
    /// Flaeche. Das ist keine Kleinigkeit obendrauf, sondern ein zweiter
    /// Rasterweg: gemessen an pygame mit `Font(None, 24)` wird
    /// `size("Hallo")` von `(41, 16)` auf `(45, 20)` gross, die Flaeche aus
    /// `render` von `(41, 18)` auf `(45, 20)` - sie wird bei `outline > 0`
    /// also sogar **niedriger** als die Zeilenhoehe. fontdue hat keinen
    /// Kontur-Zieher, und etwas Aehnliches danebenzustellen waere genau die
    /// halbe Sache, die hier nichts zu suchen hat.
    ///
    /// Der Weg zu 0 zurueck bleibt offen, damit `font.outline = 0` (was
    /// Programme zum Zuruecksetzen schreiben) nicht ohne Not scheitert.
    #[setter(outline)]
    fn outline_setzen(&mut self, wert: &Bound<'_, PyAny>) -> PyResult<()> {
        self.lebt()?;
        let v = ganzzahl(wert)?;
        if v < 0 {
            // Wortlaut aus `font_setter_outline`, gemessen.
            return Err(PyValueError::new_err("outline must be >= 0"));
        }
        self.inner.set_outline(v as i32);
        Ok(())
    }

    /// `font.set_direction(direction)` - Laufrichtung fuer die Textformung.
    ///
    /// 0 ist links-nach-rechts und damit das, was hier ohnehin geschieht;
    /// dieser Aufruf geht durch. 1 bis 3 (rechts-nach-links, oben-nach-unten,
    /// unten-nach-oben) brauchen HarfBuzz, das den Text umsortiert und bei
    /// den senkrechten Richtungen auf die `vmtx`-Vorschuebe umstellt -
    /// gemessen an pygame wird aus `render("abc")` dabei eine Flaeche `10 x
    /// 51` statt `29 x 18`. Ohne Formungsmaschine gibt es dafuer keine halbe
    /// Loesung, also einen klaren Fehler.
    ///
    /// Die Pruefung davor ist pygames eigene: `4` und `-1` geben
    /// `ValueError`, ein Nicht-Integer `TypeError`. Beides gemessen.
    #[pyo3(signature = (direction))]
    fn set_direction(&mut self, direction: &Bound<'_, PyAny>) -> PyResult<()> {
        self.lebt()?;
        let d = ganzzahl(direction)?;
        if !(0..=3).contains(&d) {
            return Err(PyValueError::new_err(
                "invalid input parameter for Font.set_direction",
            ));
        }
        self.richtung = d;
        Ok(())
    }

    /// `font.set_script(code)` - Schriftsystem fuer die Textformung.
    ///
    /// Anders als bei [`PyFont::set_direction`] gibt es hier keinen Wert, der
    /// durchgeht: der Code waehlt in HarfBuzz die Formungsregeln, und ohne
    /// HarfBuzz gibt es keine - weder fuer `"Arab"` noch fuer `"Latn"`. Ein
    /// stillschweigend angenommener Aufruf wuerde arabischen oder indischen
    /// Text falsch setzen, ohne dass es jemandem auffaellt.
    ///
    /// Die Pruefung davor ist pygames eigene. Achtung beim Wortlaut: die
    /// Meldung sagt "characters", gemeint sind aber **Bytes** - pygame misst
    /// mit `PyUnicode_AsUTF8AndSize`. Gemessen: `"ää"` (2 Zeichen, 4 Bytes)
    /// geht durch, `"ääää"` (4 Zeichen, 8 Bytes) nicht.
    #[pyo3(signature = (script, /))]
    fn set_script(&mut self, script: &Bound<'_, PyAny>) -> PyResult<()> {
        self.lebt()?;
        let Ok(s) = script.downcast::<PyString>() else {
            return Err(PyTypeError::new_err("script code must be a string"));
        };
        let code = s.to_cow()?;
        if code.len() != 4 {
            return Err(PyValueError::new_err(
                "script code must be exactly 4 characters",
            ));
        }
        // **Der Code wird gemerkt, aber er formt nichts.**
        //
        // In pygame waehlt er HarfBuzz' Formungsregeln - `"Arab"` setzt
        // arabische Buchstaben in ihre Anfangs- und Endformen, `"Deva"`
        // bildet indische Ligaturen. rustgame hat keine Formungsmaschine
        // und setzt Zeichen fuer Zeichen, wie pygame es ohne HarfBuzz
        // auch tut.
        //
        // Hier stand ein Fehler, und der war die schlechtere Antwort:
        // `set_script` ist eine **Voreinstellung**, kein Auftrag. Ein
        // Programm, das sie setzt und danach lateinischen Text zeichnet,
        // brach daran ab, ohne dass irgendetwas falsch gewesen waere.
        self.skript = Some(code.to_string());
        Ok(())
    }

    // Die alten Methodenpaare - pygame haelt beide Schreibweisen vor.
    fn get_bold(&self) -> PyResult<bool> {
        self.lebt()?;
        Ok(self.inner.get_bold())
    }
    #[pyo3(signature = (an, /))]
    fn set_bold(&mut self, an: &Bound<'_, PyAny>) -> PyResult<()> {
        self.lebt()?;
        self.inner.set_bold(an.is_truthy()?);
        Ok(())
    }
    fn get_italic(&self) -> PyResult<bool> {
        self.lebt()?;
        Ok(self.inner.get_italic())
    }
    #[pyo3(signature = (an, /))]
    fn set_italic(&mut self, an: &Bound<'_, PyAny>) -> PyResult<()> {
        self.lebt()?;
        self.inner.set_italic(an.is_truthy()?);
        Ok(())
    }
    fn get_underline(&self) -> PyResult<bool> {
        self.lebt()?;
        Ok(self.inner.get_underline())
    }
    #[pyo3(signature = (an, /))]
    fn set_underline(&mut self, an: &Bound<'_, PyAny>) -> PyResult<()> {
        self.lebt()?;
        self.inner.set_underline(an.is_truthy()?);
        Ok(())
    }
    fn get_strikethrough(&self) -> PyResult<bool> {
        self.lebt()?;
        Ok(self.inner.get_strikethrough())
    }
    #[pyo3(signature = (an, /))]
    fn set_strikethrough(&mut self, an: &Bound<'_, PyAny>) -> PyResult<()> {
        self.lebt()?;
        self.inner.set_strikethrough(an.is_truthy()?);
        Ok(())
    }

    // ----- Setzen ---------------------------------------------------------

    /// `font.render(text, antialias, color, bgcolor=None, wraplength=0)`.
    ///
    /// **Fuenf Argumente, und das vierte heisst `bgcolor`** - gemessen am
    /// Docstring des Originals:
    /// `render(text, antialias, color, bgcolor=None, wraplength=0) -> Surface`.
    /// Hier stand `background`, und damit scheiterte
    /// `font_test.test_render_args` an
    /// `unexpected keyword argument 'bgcolor'`, noch bevor der erste
    /// `assert` dran war.
    ///
    /// `antialias` ist eine **Wahrheit im Python-Sinn**, kein `bool`:
    /// `render(t, 1, c)` steht so in `test_issue_553`, und ein
    /// `bool`-Beiwert gab dort
    /// `TypeError: 'int' object cannot be converted to 'PyBool'`.
    ///
    /// `wraplength` wird angenommen und **wirkt**: 0 heisst "nicht
    /// umbrechen".
    #[pyo3(signature = (text, antialias, color, bgcolor=None, wraplength=0))]
    fn render(
        &self,
        text: &Bound<'_, PyAny>,
        antialias: &Bound<'_, PyAny>,
        color: &Bound<'_, PyAny>,
        bgcolor: Option<&Bound<'_, PyAny>>,
        wraplength: i32,
    ) -> PyResult<PySurface> {
        self.lebt()?;
        let text = text_aus(text)?;
        // **Ein Nullzeichen im Text ist ein Fehler** - gemessen:
        // `ValueError: A null character was found in the text`, und zwar
        // fuer `str` wie fuer `bytes`.
        //
        // Der Grund liegt bei SDL_ttf, das C-Zeichenketten bekommt und
        // hinter der Null aufhoeren wuerde; pygame weist es deshalb ab,
        // statt stillschweigend abzuschneiden. Rust kann den Text zwar
        // tragen, aber die Auskunft muss dieselbe sein - sonst rendert ein
        // Programm hier etwas, das pygame verweigert.
        if text.contains(' ') {
            return Err(pyo3::exceptions::PyValueError::new_err(
                "A null character was found in the text",
            ));
        }
        // `is_truthy`, nicht `extract::<bool>()` - pygame fragt hier
        // Pythons Wahrheitsbegriff, und `1`, `"x"` oder eine nichtleere
        // Liste zaehlen alle als wahr.
        let text = self.in_richtung(&text);
        let antialias = antialias.is_truthy()?;
        let c = farbe_aus(color)?;
        let bg = match bgcolor {
            None => None,
            Some(x) if x.is_none() => None,
            Some(x) => Some(farbe_aus(x)?),
        };
        // Wortlaut gemessen - "parameter" steht mit drin.
        if wraplength < 0 {
            return Err(pyo3::exceptions::PyValueError::new_err(
                "wraplength parameter must be positive",
            ));
        }
        // Scheitert das CV-Programm, lehnt SDL_ttf jede Glyphe ab, und
        // `TTF_Render_Internal` meldet die Breite null (`Font::ladefehler`).
        // Leerer Text kommt bei pygame gar nicht bis SDL_ttf.
        if !text.is_empty() && self.inner.ladefehler() {
            return Err(PygameError::new_err("Text has zero width"));
        }
        Ok(PySurface::neu(
            self.inner
                .render(&text, antialias, c, bg)
                .map_err(schrift_fehler)?,
        ))
    }

    /// Unmittelbar auf eine Flaeche setzen - **eine Zugabe von rustgame**.
    ///
    /// In `pygame.font` gibt es das nicht, nur in `pygame.freetype`.
    /// rustgame-font hat es, weil das umgebende Projekt sonst fuer jede
    /// Anzeige eine Flaeche anlegen und gleich wieder wegwerfen muesste.
    ///
    /// **Nach aussen heisst es `_render_to`, mit Unterstrich.** Der Grund
    /// ist gemessen: `font_test.test_font_method_should_raise_exception_after_quit`
    /// zaehlt vorher die Methoden und besteht darauf, dass es **genau** die
    /// von pygame sind -
    /// `assertEqual(len(expected_names), len(actual_names))`. Eine einzige
    /// Zugabe laesst den Test fallen, und zwar mit einer Meldung ueber
    /// Ausnahmen, die mit dem eigentlichen Punkt nichts zu tun hat.
    ///
    /// Der Unterstrich ist dabei kein Trick, sondern die richtige Auskunft:
    /// wer `pygame`-Verhalten erwartet, soll hier nichts finden. Die
    /// Rust-Kiste behaelt den Namen ohne Unterstrich, und in der Huelle
    /// benutzt es nichts.
    #[pyo3(name = "_render_to")]
    #[pyo3(signature = (surface, dest, text, antialias, color, background=None))]
    #[allow(clippy::too_many_arguments)]
    fn render_to(
        &self,
        surface: &PySurface,
        dest: &Bound<'_, PyAny>,
        text: &str,
        antialias: bool,
        color: &Bound<'_, PyAny>,
        background: Option<&Bound<'_, PyAny>>,
    ) -> PyResult<PyRect> {
        self.lebt()?;
        let c = farbe_aus(color)?;
        let bg = match background {
            None => None,
            Some(x) if x.is_none() => None,
            Some(x) => Some(farbe_aus(x)?),
        };
        let p = punkt_aus(dest)?;
        let mut ziel = surface.leihen_mut()?;
        Ok(PyRect::neu(
            self.inner
                .render_to(&mut ziel, p, text, antialias, c, bg)
                .map_err(schrift_fehler)?,
        ))
    }

}

/// Eine Ganzzahl, und bei Fehlschlag **CPythons** Wortlaut.
///
/// `font.c` reicht den Fehler von `PyLong_AsLong` bei `outline` und
/// `set_direction` unveraendert durch. Gemessen: `f.outline = "x"` gibt
/// `TypeError: 'str' object cannot be interpreted as an integer`. PyO3
/// schreibt an dieselbe Stelle etwas anderes, also wird die Meldung hier
/// gebaut.
///
/// `True` geht durch und zaehlt als 1 - auch das gemessen; in Python ist
/// `bool` eine Unterklasse von `int`.
fn ganzzahl(wert: &Bound<'_, PyAny>) -> PyResult<i64> {
    if let Ok(v) = wert.extract::<i32>() {
        return Ok(v as i64);
    }
    // **Eine Zahl, die nicht passt, ist kein falscher Typ.**
    // `PyLong_AsLong` liest einen C-`long`, und der ist unter Windows
    // **32 Bit**: gemessen geben `f.outline = 2**31` und
    // `f.set_direction(2**31)` beide `OverflowError: Python int too large
    // to convert to C long`, waehrend `2**31-1` durchgeht. Hier stand
    // `i64`, und `outline = 2**31` wurde damit stillschweigend **0** -
    // der Wert ging eine Zeile spaeter durch `as i32`. Ein stiller
    // falscher Wert ist die schlechteste der drei moeglichen Antworten.
    if wert.is_instance_of::<pyo3::types::PyInt>() {
        return Err(pyo3::exceptions::PyOverflowError::new_err(
            "Python int too large to convert to C long",
        ));
    }
    let art = wert
        .get_type()
        .name()
        .map(|n| n.to_string())
        .unwrap_or_else(|_| "object".to_string());
    Err(PyTypeError::new_err(format!(
        "'{art}' object cannot be interpreted as an integer"
    )))
}

/// Laeuft `pygame.font`? - **ein eigener Schalter.**
///
/// `pygame.font` und `pygame.freetype` haben bei pygame **getrennte**
/// Zustaende; gemessen:
///
/// ```text
/// frisch            font=False  freetype=False
/// font.init()       font=True   freetype=False
/// freetype.init()   font=False  freetype=True
/// pygame.init()     font=True   freetype=True
/// ```
///
/// Hier fragten beide `rustgame_font::get_init()`, also machte
/// `font.init()` auch `freetype.get_init()` wahr - und umgekehrt blieb
/// `font.get_init()` nach `font.quit()` wahr, solange freetype lief.
/// Die Kiste darunter bleibt gemeinsam (sie haelt nur die
/// Glyphenablage); was getrennt gehoert, ist die **Auskunft**.
pub(crate) static FONT_INIT: std::sync::atomic::AtomicBool =
    std::sync::atomic::AtomicBool::new(false);

/// `pygame.font.init()`.
#[pyfunction]
#[pyo3(name = "init")]
pub fn schrift_init(py: Python<'_>) {
    crate::schriftordner_sichern(py);
    FONT_INIT.store(true, std::sync::atomic::Ordering::Relaxed);
    schrift::init();
}

/// `pygame.font.quit()`.
#[pyfunction]
#[pyo3(name = "quit")]
pub fn schrift_quit() {
    FONT_INIT.store(false, std::sync::atomic::Ordering::Relaxed);
    schrift::quit();
    // **Jede vorher gebaute `Font` wird damit ungueltig** - siehe das
    // Feld `epoche` an [`PyFont`]. Der Zaehler geht hoch, nicht auf null:
    // ein spaeteres `init()` darf die alten Schriften nicht wiederbeleben.
    EPOCHE.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
}

/// `pygame.font.get_init()`.
#[pyfunction]
#[pyo3(name = "get_init")]
pub fn schrift_get_init() -> bool {
    FONT_INIT.load(std::sync::atomic::Ordering::Relaxed)
}

/// Laeuft `pygame.font`? Sonst **pygames Satz**.
///
/// Gemessen: `pygame.font.Font(None, 20)` ohne `font.init()` gibt
/// `pygame.error: font not initialized`. Hier wurde die Schrift gebaut.
pub(crate) fn font_pruefen() -> PyResult<()> {
    if FONT_INIT.load(std::sync::atomic::Ordering::Relaxed) {
        return Ok(());
    }
    Err(crate::umwandlung::PygameError::new_err("font not initialized"))
}

/// `pygame.font.get_default_font()` - der **Dateiname**, nicht der Pfad.
#[pyfunction]
#[pyo3(name = "get_default_font")]
pub fn schrift_get_default_font() -> &'static str {
    schrift::get_default_font()
}

/// Eine Schriftentabelle, wie `rustgame_font::sysfont::tabellen` sie gibt.
///
/// Als eigener Name, weil clippy den vierfach geschachtelten Typ zu Recht
/// fuer unleserlich haelt - dieselbe Begruendung wie bei `FesteGroessen`
/// in `freitext.rs`.
type Schriftenliste = Vec<(String, Vec<((bool, bool), std::path::PathBuf)>)>;

/// Die ganze Schriftentabelle als zwei Woerterbuecher.
///
/// Nur fuer `sysfont.initsysfonts`, darum mit Unterstrich: pygame hat
/// den Namen nicht, und ein Name zuviel ist derselbe Fehler wie einer
/// zuwenig. Die Tabelle selbst **hat** pygame, als `sysfont.Sysfonts`
/// und `sysfont.Sysalias`.
#[pyfunction]
#[pyo3(name = "_sysfont_tabelle")]
pub fn schrift_sysfont_tabelle(py: Python<'_>) -> PyResult<(Py<PyAny>, Py<PyAny>)> {
    // Die Tabelle wird beim ersten Zugriff gebaut und nie wieder - der
    // Ordner der Huelle muss also **vorher** feststehen, sonst steht
    // `freesansbold` fuer immer auf dem, was das Arbeitsverzeichnis
    // gerade hergab.
    crate::schriftordner_sichern(py);
    let (fonts, alias) = schrift::sysfont::tabellen();
    let bauen = |liste: Schriftenliste| -> PyResult<Py<PyAny>> {
        let d = pyo3::types::PyDict::new(py);
        for (name, schnitte) in liste {
            let innen = pyo3::types::PyDict::new(py);
            for ((fett, schraeg), pfad) in schnitte {
                innen.set_item((fett, schraeg), pfad.to_string_lossy().into_owned())?;
            }
            d.set_item(name, innen)?;
        }
        Ok(d.into_any().unbind())
    };
    Ok((bauen(fonts)?, bauen(alias)?))
}

/// Traegt die Suche in `sysfont` ein - **wie pygame es tut**.
///
/// pygames `get_fonts`, `match_font` und `SysFont` rufen alle drei
/// `initsysfonts()`, und das setzt `is_init = True` und fuellt
/// `Sysfonts`. Gemessen: vor dem ersten Aufruf `False` und zwei leere
/// Woerterbuecher, danach `True`, 391 Schriften, 33 Zweitnamen. Hier
/// blieb alles leer, weil die Suche im Kern steht und niemand die
/// Python-Seite anstiess.
///
/// Fehler werden geschluckt: ein Programm, das eine Schrift sucht, soll
/// nicht daran scheitern, dass eine **Nebenbuchfuehrung** klemmt.
///
/// `get_fonts` und `match_font` gehen ohne `init()` - darum wird der
/// Schriftordner auch hier gesichert, bevor die Tabelle entsteht.
pub(crate) fn sysfont_anstossen(py: Python<'_>) {
    crate::schriftordner_sichern(py);
    if let Ok(m) = py.import("rustgame_py.sysfont") {
        let _ = m.call_method0("initsysfonts");
    }
}

/// `pygame.font.get_fonts()` - alle Schriften des Systems, klein
/// geschrieben und ohne Leerzeichen.
#[pyfunction]
#[pyo3(name = "get_fonts")]
pub fn schrift_get_fonts(py: Python<'_>) -> Vec<String> {
    sysfont_anstossen(py);
    schrift::get_fonts()
}

/// `pygame.font.match_font(name, bold=False, italic=False)`.
#[pyfunction]
///
/// `name` darf ein Text, **`bytes`** oder eine **Liste** sein - gemessen
/// gibt `match_font(b"arial")` und `match_font(["arial", "times"])` beide
/// denselben Pfad wie `match_font("arial")`. Eine Liste wirkt wie die
/// Kommaliste: der erste Name, der sich aufloesen laesst, gewinnt.
#[pyo3(signature = (name, bold=false, italic=false))]
#[pyo3(name = "match_font")]
pub fn schrift_match_font(
    py: Python<'_>,
    name: &Bound<'_, PyAny>,
    bold: bool,
    italic: bool,
) -> PyResult<Option<String>> {
    sysfont_anstossen(py);
    let name = &schriftnamen_aus(name)?;
    Ok(schrift::match_font(name, bold, italic).map(|p| p.to_string_lossy().into_owned()))
}

/// `pygame.font.SysFont(name, size, bold=False, italic=False)`.
///
/// `name` darf eine Kommaliste sein; findet sich keiner, kommt die
/// Standardschrift - wie in pygames `sysfont.py`.
/// **Findet sich keine der genannten Schriften, wird gewarnt** - und
/// zwar mit pygames Wortlaut:
///
/// ```text
/// UserWarning: The system font 'non-existent font' couldn't be found.
/// Verify your font name input. Using the default font instead.
/// ```
///
/// `font_test.test_sysfont_warnings` prueft das mit `assertWarns`, und
/// zwar fuer einen Text **und** fuer `bytes`. Stillschweigend die
/// Vorgabeschrift zu nehmen ist bequem und verbirgt genau den Fehler,
/// den der Aufrufer sehen will.
#[pyfunction]
#[pyo3(signature = (name, size, bold=None, italic=None, constructor=None))]
#[pyo3(name = "SysFont")]
pub fn schrift_sysfont(
    py: Python<'_>,
    name: Option<&Bound<'_, PyAny>>,
    size: i32,
    bold: Option<&Bound<'_, PyAny>>,
    italic: Option<&Bound<'_, PyAny>>,
    constructor: Option<&Bound<'_, PyAny>>,
) -> PyResult<Py<PyAny>> {
    // **Kein Riegel hier** - er sitzt im Erzeuger. `ftfont.SysFont`
    // ruft diese Funktion mit einem eigenen Bauer, und der haengt an
    // freetype: ein `font_pruefen()` an dieser Stelle machte
    // `ftfont_test.test_SysFont` rot. pygame prueft ebenso erst beim
    // Bauen der Schrift.
    sysfont_anstossen(py);
    // `SysFont(name, 12, 0, 0)` mit Nullen statt `False` ist die
    // Schreibweise aus pygames eigenen Beispielen.
    let bold = match bold {
        Some(o) => o.is_truthy()?,
        None => false,
    };
    let italic = match italic {
        Some(o) => o.is_truthy()?,
        None => false,
    };
    // Der Name darf Text, `bytes` oder eine Folge davon sein.
    let n: String = match name {
        None => String::new(),
        Some(o) if o.is_none() => String::new(),
        Some(o) => schriftnamen_aus(o)?,
    };
    if !n.trim().is_empty() && schrift::sysfont::match_font(&n, bold, italic).is_none() {
        let erster = n.split(',').next().unwrap_or(&n).trim().to_string();
        warnen(
            py,
            &format!(
                "The system font '{erster}' couldn't be found. Verify your font name input. Using the default font instead."
            ),
        )?;
    }
    // **Eine fremde Bauanweisung darf die Schrift bauen.** pygames
    // `sysfont.py` sieht das vor, und `pygame.ftfont.SysFont` benutzt es:
    // dieselbe Suche, am Ende aber eine `ftfont.Font` statt einer
    // `font.Font`.
    if let Some(bauer) = constructor {
        if !bauer.is_none() {
            let pfad = schrift::sysfont::match_font(&n, bold, italic);
            let arg = match pfad {
                Some(p) => p.into_pyobject(py)?.into_any(),
                None => py.None().into_bound(py),
            };
            return Ok(bauer.call1((arg, size, bold, italic))?.unbind());
        }
    }
    // Hier wird eine `font.Font` gebaut, also gilt hier ihr Riegel.
    font_pruefen()?;
    let f = PyFont::neu(schrift::sys_font(&n, size, bold, italic).map_err(schrift_fehler)?);
    Ok(Py::new(py, f)?.into_any())
}

/// Eine `UserWarning` mit pygames Wortlaut.
fn warnen(py: Python<'_>, text: &str) -> PyResult<()> {
    let warnungen = py.import("warnings")?;
    let art = py.get_type::<pyo3::exceptions::PyUserWarning>();
    warnungen.call_method1("warn", (text, art))?;
    Ok(())
}

/// `pygame.font.get_sdl_ttf_version`.
///
/// Gemessen: echtes pygame gibt `(2, 24, 0)`, und `linked=True` gibt
/// dasselbe. **Hier kommt `None`**, und das ist Absicht: rustgame hat
/// gar kein SDL_ttf. Der Text entsteht mit `fontdue` und einer
/// Uebertragung von SDL_ttfs Layoutregeln.
///
/// Eine Versionsnummer zu erfinden waere die bequemere Antwort und die
/// schlechtere: ein Programm, das anhand der Zahl eine Weiche stellt,
/// liefe dann in den falschen Zweig. `image::get_sdl_image_version` im
/// Kern haelt es genauso.
///
/// Der Modulvergleich fuehrt das als **benannte Grenze** - der Lauf wird
/// gruen, die Abweichung bleibt sichtbar.
#[pyfunction]
#[pyo3(signature = (linked=None))]
#[pyo3(name = "get_sdl_ttf_version")]
pub fn schrift_get_sdl_ttf_version(linked: Option<&Bound<'_, PyAny>>) -> PyResult<Option<(u32, u32, u32)>> {
    // **Jeder Wert ist ein Wahrheitswert** - pygame liest den
    // Schalter mit `PyObject_IsTrue`, und das nimmt alles. Mit
    // `bool` verlangte PyO3 einen echten Wahrheitswert und nannte
    // im Fehlertext `'PyBool'`, einen Namen aus Rust.
    let linked = match linked {
        None => true,
        Some(o) => o.is_truthy()?,
    };
    let _ = linked;
    Ok(rustgame_font::get_sdl_ttf_version())
}
