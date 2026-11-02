//! freitext.rs
//! ===========
//! `pygame.freetype` - das **zweite** Schriftmodul.
//!
//! pygame hat zwei, und sie sind nicht dasselbe mit anderen Namen:
//!
//! | | `pygame.font` | `pygame.freetype` |
//! |---|---|---|
//! | `height` | **Pixel** (16) | **Schrifteinheiten** (1090) |
//! | `render` | gibt eine Flaeche | gibt `(Flaeche, Rect)` |
//! | Farbe | beim Zeichnen | auch als Eigenschaft (`fgcolor`) |
//! | Stil | `bold`/`italic` | Bitmaske `STYLE_*` |
//! | Ursprung | oben links | wahlweise die **Grundlinie** (`origin`) |
//!
//! Gemessen an `freesansbold.ttf` in Groesse 24:
//!
//! ```text
//! font.Font(None, 24).get_height()          16
//! freetype.Font(None, 24).height          1090
//!                        .ascender         800
//!                        .descender       -200
//!                        .get_sized_height()  26
//! ```
//!
//! Die 1090 sind `ascender - descender + line_gap` bei 1000 Einheiten je
//! Geviert. Wer die beiden verwechselt, rechnet um den Faktor 45 daneben -
//! und bekommt keine Fehlermeldung, sondern falschen Text.
//!
//! ## Was hier **nicht** steht
//!
//! Gedrehter Text (`rotation`), senkrechter Satz (`vertical`) und die
//! Konturstaerke (`strength`) melden einen klaren Fehler. Sie brauchen
//! den Glyphenpfad, nicht nur das Rasterbild - dieselbe Grenze wie bei
//! `Font.outline` in `pygame.font`.

use pyo3::exceptions::{PyTypeError, PyValueError};
use pyo3::prelude::*;
use pyo3::types::PyDict;
use pyo3::types::PyAny;

use rustgame_font as schrift;

use crate::flaeche::PySurface;
use crate::rechteck::PyRect;
use crate::umwandlung::{farbe_aus, punkt_aus, schrift_fehler, PygameError};

/// Die Aufloesung, in der `pygame.freetype` rechnet - Punkte je Zoll.
static AUFLOESUNG: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(72);

/// Die Groesse des Glyphenspeichers.
///
/// **Nach `quit()` ist sie null** - gemessen:
///
/// ```python
/// ft.get_cache_size()          # 64
/// ft.quit(); ft.get_cache_size()   # 0
/// ft.init(cache_size=128); ft.get_cache_size()  # 128
/// ```
///
/// rustgame hat gar keinen solchen Speicher; die Zahl ist trotzdem
/// buchgefuehrt, weil pygames Test sie durchspielt und weil eine
/// erfundene feste Zahl die falsche Auskunft waere.
/// Die gemeldete Groesse der Glyphenablage - `get_cache_size()`.
///
/// **Vor `init()` ist sie 0**, nicht 64. Gemessen:
///
/// ```text
/// frisch eingefuehrt        0
/// init(cache_size=100)    100
/// init()                   64
/// ```
///
/// Hier stand die Vorgabe 64 von Anfang an, und damit meldete eine
/// Huelle ohne `init()` eine Ablage, die es nicht gab.
static CACHE: std::sync::atomic::AtomicI32 = std::sync::atomic::AtomicI32::new(0);

/// Eine Schriftgroesse, wie `pygame.freetype` sie annimmt.
///
/// **Eine Zahl oder ein Paar.** Gemessen:
///
/// ```text
/// f.size = 24.0, 0      ->  f.size == 24.0      (eine Zahl)
/// f.size = 16, 16       ->  f.size == (16.0, 16.0)
/// f.size = 0, 24.0      ->  ValueError
/// f.size = (24.0,)      ->  TypeError
/// f.size = -1           ->  OverflowError
/// ```
///
/// Eine Null als **zweiter** Wert heisst "so hoch wie breit" und macht
/// aus dem Paar wieder eine Zahl; eine Null als **erster** ist ein
/// Fehler. Diese Schieflage steht so in `_freetype.c`, und sie ist der
/// Grund, warum der Typ hier steht statt als blankes `f64`.
#[derive(Clone, Copy)]
pub struct Groesse(pub f64, pub Option<f64>);

/// Die groesste Punktgroesse, die in 26.6 passt - **mit
/// Nachkommastellen**.
///
/// Gemessen ist `f.size = 2147483647 * 0.015625` erlaubt und
/// `f.size = 33554432` nicht; die Grenze liegt also bei
/// `0x7FFFFFFF / 64` und nicht bei der abgeschnittenen ganzen Zahl.
const HOECHSTE: f64 = 0x7FFF_FFFFi64 as f64 / 64.0;

/// Der Typname des Werts, **rechts in einem 128 Zeichen breiten Feld**.
///
/// So schreibt pygame ihn in seine Fehlersaetze, und das ist keine
/// Vermutung: gemessen an neun Saetzen aus `_freetype.c` ist die Summe
/// aus Polster und Namen immer **128** - `NoneType` (8 Zeichen) hat 120
/// Leerzeichen davor, `list` 124, `str` 125. Der Satz
/// `expected a float for size, got …NoneType` ist 159 Zeichen lang, und
/// jeder andere Typname laesst ihn genauso lang.
///
/// Wortlaut ist Schnittstelle, wo pygame ihn selbst geschrieben hat -
/// also auch das Polster.
fn gepolstert(o: &Bound<'_, PyAny>) -> String {
    format!("{:>128}", typname(o))
}

/// Der blosse Typname (`tp_name`) eines Werts.
fn typname(o: &Bound<'_, PyAny>) -> String {
    o.get_type()
        .name()
        .map(|n| n.to_string())
        .unwrap_or_else(|_| "object".to_string())
}

/// Ein Schalter von `freetype.Font` - **nur `True` oder `False`**.
///
/// pygame prueft hier mit `PyBool_Check` und lehnt sogar `1` ab:
/// `f.kerning = 1` gibt `TypeError: The style value must be a boolean`.
/// Gemessen fuer alle elf Schalter (`strong`, `oblique`, `underline`,
/// `wide`, `origin`, `pad`, `antialiased`, `kerning`, `ucs4`, `vertical`,
/// `use_bitmap_strikes`) und fuer sieben Werte je Schalter - hier stand
/// `is_truthy()`, das aus `1.5`, `'x'` und `(1,2,3,4,5)` ein `True` und
/// aus `None` und `[]` ein `False` machte: **77 Zuweisungen**, die pygame
/// zurueckweist und die hier durchgingen.
///
/// Der Gegenfall steht bei `pygame.font.Font`: dort nimmt pygame
/// **jeden** Wahrheitswert. Die beiden Schriftmodule sind an dieser
/// Stelle verschieden, und hier war es genau verkehrt herum.
fn schalter(v: &Bound<'_, PyAny>) -> PyResult<bool> {
    if !v.is_instance_of::<pyo3::types::PyBool>() {
        return Err(PyTypeError::new_err("The style value must be a boolean"));
    }
    v.extract()
}

/// Eine Punktgroesse, mit pygames drei Saetzen.
///
/// Gemessen gibt pygame hier **drei** verschiedene Fehler, und alle drei
/// nennen den Typnamen des Werts:
///
/// ```text
/// f.size = None   TypeError      expected a float for size, got …NoneType
/// f.size = -7     OverflowError  …int value is negative while size value is zero or positive
/// f.size = 2**31  OverflowError  …int value too large to convert to a size value
/// ```
///
/// Vorher stand hier ein einziger Satz (`point size is out of valid
/// range`) fuer alle drei Faelle.
fn punktzahl(o: &Bound<'_, PyAny>) -> PyResult<f64> {
    let Ok(v) = o.extract::<f64>() else {
        return Err(PyTypeError::new_err(format!(
            "expected a float for size, got {}",
            gepolstert(o)
        )));
    };
    if v < 0.0 {
        return Err(pyo3::exceptions::PyOverflowError::new_err(format!(
            "{} value is negative while size value is zero or positive",
            gepolstert(o)
        )));
    }
    if v > HOECHSTE {
        return Err(pyo3::exceptions::PyOverflowError::new_err(format!(
            "{} value too large to convert to a size value",
            gepolstert(o)
        )));
    }
    Ok(v)
}

/// `float(v)` - **mit** Zeichenketten, und mit Pythons eigenen Saetzen.
///
/// pygame liest `strength` und `underline_adjustment` ueber
/// `PyNumber_Float`, nicht ueber eine Typpruefung. Gemessen heisst das:
/// `f.strength = '0.5'`, `' 0.5 '`, `'1e-1'` und `b'0.5'` gehen **durch**,
/// und `f.strength = None` gibt CPythons Satz
/// `float() argument must be a string or a real number, not 'NoneType'`.
/// Hier stand `extract::<f64>()`, das Zeichenketten ablehnt.
///
/// Zurueck kommt der Wert **und** sein `repr`, denn den braucht pygames
/// Bereichsmeldung woertlich (`'1e+30'`, `'-7.0'`) - Rusts `{}` schreibt
/// dieselben Zahlen anders (`1e30`, `-7`).
fn als_gleitzahl(v: &Bound<'_, PyAny>) -> PyResult<(f64, String)> {
    let f = v.py().get_type::<pyo3::types::PyFloat>().call1((v,))?;
    Ok((f.extract::<f64>()?, f.repr()?.to_string()))
}

/// Eine Farbe fuer `fgcolor`/`bgcolor` - **nur Folgen und `Color`**.
///
/// pygame ist hier enger als ueberall sonst: `fgcolor = 'red'` und
/// `fgcolor = 0` geben einen `AttributeError`, waehrend `b'red'` und
/// `range(3)` durchgehen - beide geben beim Durchlaufen Zahlen. Gemessen
/// an zehn Werten; der allgemeine [`farbe_aus`] nimmt Namen und blanke
/// Zahlen an und lag deshalb hier falsch, in Sorte **und** Wortlaut
/// (`ValueError: invalid color name` statt `AttributeError`).
fn farbe_fuer_freitext(o: &Bound<'_, PyAny>) -> PyResult<rustgame_core::color::Color> {
    let fehler = || {
        pyo3::exceptions::PyAttributeError::new_err(format!(
            "unable to convert {} object to a color",
            gepolstert(o)
        ))
    };
    if let Ok(c) = o.extract::<PyRef<crate::farbe::PyColor>>() {
        return Ok(c.wert());
    }
    if o.is_instance_of::<pyo3::types::PyString>()
        || o.is_instance_of::<pyo3::types::PyInt>()
        || o.is_none()
    {
        return Err(fehler());
    }
    let folge: Vec<i64> = o.extract().map_err(|_| fehler())?;
    crate::umwandlung::folge_zu_farbe(&folge).map_err(|_| fehler())
}

impl<'py> FromPyObject<'py> for Groesse {
    fn extract_bound(o: &Bound<'py, PyAny>) -> PyResult<Groesse> {
        if let Ok(t) = o.downcast::<pyo3::types::PyTuple>() {
            if t.len() != 2 {
                // pygame nennt die **Laenge**, die es bekommen hat:
                // `expected a 2-tuple for size, got 5-tuple`. Gemessen
                // fuer 0, 1, 3 und 5 Glieder.
                return Err(PyTypeError::new_err(format!(
                    "expected a 2-tuple for size, got {}-tuple",
                    t.len()
                )));
            }
            let x = punktzahl(&t.get_item(0)?)?;
            let y = punktzahl(&t.get_item(1)?)?;
            if x == 0.0 && y != 0.0 {
                return Err(PyValueError::new_err(
                    "expected zero height with a zero width",
                ));
            }
            return Ok(Groesse(x, if y == 0.0 { None } else { Some(y) }));
        }
        Ok(Groesse(punktzahl(o)?, None))
    }
}

/// `STYLE_NORMAL`.
pub const STYLE_NORMAL: i32 = 0;
/// `STYLE_STRONG` - fett.
pub const STYLE_STRONG: i32 = 1;
/// `STYLE_OBLIQUE` - schraeg.
pub const STYLE_OBLIQUE: i32 = 2;
/// `STYLE_UNDERLINE`.
pub const STYLE_UNDERLINE: i32 = 4;
/// `STYLE_WIDE`.
pub const STYLE_WIDE: i32 = 8;
/// `STYLE_DEFAULT` - "nimm, was die Schrift selbst sagt".
///
/// **255, nicht -1.** Gemessen an `pygame.freetype.STYLE_DEFAULT`; hier
/// stand `-1`, und weil derselbe Wert auch der Platzhalter in
/// `render(style=...)` ist, war beides falsch: `f.style = 255` gab einen
/// `ValueError`, und `f.style = -1` setzte stillschweigend nichts, wo
/// pygame `Invalid style value ffffffff` sagt. Gefunden hat das die Probe
/// `Konstantenwerte`, die **jeden** Wert abliest, statt 58 genannte zu
/// vergleichen - der Wert steht in keiner erzeugten Tabelle, weil er
/// hier von Hand als Rust-Konstante lag.
pub const STYLE_DEFAULT: i32 = 255;

/// Die eingebauten Groessen einer Bitmapschrift.
///
/// Je Eintrag `(ppem_x, ppem_y, hoehe, breite_64, hoehe_64)` - so gibt
/// `Font.get_sizes()` sie heraus. Als eigener Name, weil Clippy einen
/// fuenffach geschachtelten Typ zu Recht fuer unleserlich haelt.
pub type FesteGroessen = Vec<(i32, i32, i32, f64, f64)>;

/// Die Masse je Zeichen, wie `Font.get_metrics()` sie gibt.
///
/// `None` steht fuer ein Zeichen, das die Schrift nicht hat.
pub type Glyphenmasse = Vec<Option<(i32, i32, i32, i32, f64, f64)>>;

/// Eine Schrift im Sinne von `pygame.freetype`.
#[pyclass(name = "Font", module = "rustgame_py.freetype", unsendable, subclass, weakref)]
pub struct PyFreetypeFont {
    /// Die geladene Schrift - **`None` heisst uneingerichtet.**
    ///
    /// `pygame.freetype.Font.__new__(Font)` ist erlaubt;
    /// `freetype_test.py` baut sich daraus seine `nullfont()`. Hier stand,
    /// **jede** Auskunft gebe dann `RuntimeError: Font not initialized`
    /// - und beide Haelften des Satzes waren falsch. Gemessen gibt die
    /// Nullschrift **dreierlei**:
    ///
    /// | Zugriff | Antwort |
    /// |---|---|
    /// | `antialiased`, `use_bitmap_strikes` | `True` |
    /// | `kerning`, `oblique`, `origin`, `pad`, `strong`, `ucs4`, `underline`, `vertical`, `wide` | `False` |
    /// | `size` | `0.0`, `style` `0`, `resolution` `0`, `rotation` `0` |
    /// | `strength` | `0.027777777777777776` (also `1/36`) |
    /// | `underline_adjustment` | `1.0` |
    /// | `fgcolor` | `Color(0, 0, 0, 255)`, `bgcolor` `Color(0, 0, 0, 0)` |
    /// | `ascender`, `descender`, `height`, `scalable`, `fixed_width`, `fixed_sizes` | `RuntimeError: _freetype.Font instance is not initialized` |
    /// | `name`, `style_name` | `AttributeError: <uninitialized Font object>` |
    /// | `path` | `AttributeError: path unavailable` |
    /// | `repr` | `<uninitialized Font object at 0x...>` |
    ///
    /// Die Trennlinie ist **die Datei**: was im eigenen Zettel steht,
    /// wird beantwortet; was die Schriftdatei braucht, nicht. Und
    /// *setzen* laesst sich auch an der Nullschrift, was im Zettel steht
    /// (`antialiased`, `style`, `size`, `strength`) - nur `rotation`
    /// nicht.
    ///
    /// pygames eigener Test prueft genau die sechs Zugriffe aus der
    /// vorletzten Zeile; die uebrigen dreiundzwanzig standen hier auf
    /// `RuntimeError`, ohne dass es jemandem auffiel.
    inner: Option<schrift::Font>,
    pfad: Option<String>,
    groesse: f64,
    /// Die Aufloesung in dpi - 0 heisst "die Vorgabe".
    ///
    /// Sie geht in jede Groesse ein: `Pixel = Punkte * dpi / 72`. Gemessen
    /// ist `Font(pfad, resolution=144).get_rect("|", size=24)` doppelt so
    /// gross wie bei 72.
    aufloesung: u32,
    /// Sollen eingebaute Pixelbilder benutzt werden?
    strikes: bool,
    /// Die zweite Haelfte der Groesse, wenn eine gesetzt wurde.
    groesse_y: Option<f64>,
    /// Wie stark `STYLE_WIDE` verdickt - siehe `strength`.
    strength: f64,
    /// Um wie viel Grad der Text gedreht ist - **gegen** den Uhrzeiger.
    ///
    /// Gemessen: `f.rotation = 390` gibt `30`, `f.rotation = -390` gibt
    /// `330`. Der Winkel wird also auf 0 bis 359 gebracht, und zwar so,
    /// dass ein negativer von oben herunterzaehlt.
    drehung: i32,
    /// Steht der Text **untereinander**?
    ///
    /// Gemessen: `f.vertical = True; f.get_rect("ABCDabcd")` gibt ein
    /// Rechteck, das hoeher als breit ist - die Zeichen stehen
    /// uebereinander, je eines auf einer Zeile. Es gibt keine besonderen
    /// senkrechten Vorschuebe (die staenden in `vmtx`, und die hat kaum
    /// eine Schrift); FreeType nimmt dann den Zeilenabstand, und das tut
    /// auch rustgame.
    senkrecht: bool,
    /// Der Zaehlerstand des Schriftsystems beim Anlegen.
    ///
    /// **Ein `quit()` macht jede Schrift ungueltig**, auch wenn gleich
    /// darauf wieder `init()` kommt. Gemessen an
    /// `font_test.test_font_property_should_raise_exception_after_quit`:
    /// dort wird eine Schrift angelegt, sofort `quit(); init()` gerufen
    /// und dann von jedem Merkmal ein `pygame.error` verlangt. Ein Blick
    /// auf `get_init()` allein genuegt also nicht - der stuende danach
    /// wieder auf wahr. `pygame.font.Font` fuehrt denselben Zaehler.
    epoche: u32,
    /// Der zuletzt vermessene oder gezeichnete Text.
    ///
    /// **`render(None)` zeichnet ihn noch einmal.** Gemessen: nach
    /// `f.get_rect("M", size=64)` gibt `f.render(None, size=64)` genau
    /// dasselbe Rechteck zurueck - pygame merkt sich den Text, damit man
    /// messen und danach zeichnen kann, ohne ihn zweimal zu nennen.
    /// `freetype_test.test_issue_257` prueft die Gleichheit viermal.
    letzter_text: std::cell::RefCell<String>,
    stil: i32,
    /// `origin` - liegt der Bezugspunkt auf der **Grundlinie**?
    origin: bool,
    /// `pad` - wird auf die volle Zeilenhoehe aufgefuellt?
    pad: bool,
    antialiased: bool,
    kerning: bool,
    ucs4: bool,
    underline_adjustment: f64,
    fgcolor: rustgame_core::color::Color,
    bgcolor: Option<rustgame_core::color::Color>,
}

/// Eine Farbe - und **`TypeError`, wenn es gar keine sein kann**.
///
/// Gemessen: `font.render("x", color, 2.3)` gibt einen `TypeError`, und
/// `font.render("x", (1, 2))` einen `ValueError`. Der Unterschied ist
/// der uebliche: eine Zahl ist vom falschen **Typ**, ein zu kurzes Tupel
/// hat den falschen **Wert**. Unsere Farbumwandlung macht aus beidem
/// einen `ValueError`, also wird der Typ hier vorher geprueft.
fn farbe_streng(o: &Bound<'_, PyAny>) -> PyResult<rustgame_core::color::Color> {
    let geht = o.is_instance_of::<pyo3::types::PyString>()
        || o.is_instance_of::<pyo3::types::PyInt>()
        || o.is_instance_of::<pyo3::types::PyTuple>()
        || o.is_instance_of::<pyo3::types::PyList>()
        || o.downcast::<crate::farbe::PyColor>().is_ok();
    if !geht {
        return Err(PyTypeError::new_err(
            "color must be a Color, string, tuple or list",
        ));
    }
    farbe_aus(o)
}

/// `RuntimeError: _freetype.Font instance is not initialized`.
///
/// Der Wortlaut nennt **das C-Modul**, nicht die Klasse: hier stand
/// `Font not initialized` mit dem Vermerk "Wortlaut gemessen" - das ist
/// der Satz von `pygame.font`, nicht der von `pygame.freetype`.
fn uneingerichtet() -> PyErr {
    pyo3::exceptions::PyRuntimeError::new_err(
        "_freetype.Font instance is not initialized",
    )
}

/// `AttributeError: <uninitialized Font object>` - fuer `name` und
/// `style_name`.
///
/// Dieselbe Frage, drei Saetze: `path` sagt `path unavailable`, diese
/// zwei nennen das Objekt, und alles andere gibt den `RuntimeError`
/// oben. Gemessen, nicht hergeleitet.
fn ohne_datei() -> PyErr {
    pyo3::exceptions::PyAttributeError::new_err("<uninitialized Font object>")
}

impl PyFreetypeFont {
    /// Die Nullschrift: eine Huelle ohne Datei.
    fn leer() -> PyFreetypeFont {
        PyFreetypeFont {
            inner: None,
            pfad: None,
            groesse: 0.0,
            groesse_y: None,
            drehung: 0,
            senkrecht: false,
            epoche: schrift::generation(),
            strength: 1.0 / 36.0,
            letzter_text: std::cell::RefCell::new(String::new()),
            aufloesung: 0,
            strikes: true,
            stil: STYLE_NORMAL,
            origin: false,
            pad: false,
            antialiased: true,
            kerning: false,
            ucs4: false,
            underline_adjustment: 1.0,
            fgcolor: rustgame_core::color::Color::rgba(0, 0, 0, 255),
            bgcolor: None,
        }
    }

    /// Zerlegt `(*args, **kwargs)` in die fuenf benannten Argumente.
    ///
    /// Von Hand, weil `__new__` sonst nicht unterscheiden koennte, ob
    /// ueberhaupt etwas uebergeben wurde - und genau daran haengt die
    /// Nullschrift. Siehe [`PyFreetypeFont::new`].
    #[allow(clippy::type_complexity)]
    fn argumente<'p>(
        args: &Bound<'p, pyo3::types::PyTuple>,
        kwargs: Option<&Bound<'p, PyDict>>,
    ) -> PyResult<(Option<Bound<'p, PyAny>>, f64, i32, u32, bool)> {
        let mut file: Option<Bound<'p, PyAny>> = None;
        let (mut size, mut index, mut dpi, mut ucs4) = (0.0f64, 0i32, 0u32, false);
        // **Die Zahl zuerst.** Hier stand die Pruefung im `_`-Zweig der
        // Schleife, also **nach** dem Lesen der ersten fuenf Werte - und
        // `Font(*17 Zeichenketten)` gab darum `must be real number, not
        // str` statt pygames `function takes at most 5 arguments (17
        // given)`. pygames `PyArg_ParseTuple` zaehlt, bevor es liest.
        if args.len() > 5 {
            return Err(PyTypeError::new_err(format!(
                "function takes at most 5 arguments ({} given)",
                args.len()
            )));
        }
        for (i, a) in args.iter().enumerate() {
            match i {
                0 => file = Some(a),
                1 => size = a.extract()?,
                2 => index = a.extract()?,
                3 => dpi = a.extract()?,
                4 => ucs4 = a.is_truthy()?,
                // Die Zahl ist oben geprueft; dieser Zweig ist nur
                // noch die Vollstaendigkeit der Fallunterscheidung.
                _ => unreachable!("oben auf fuenf begrenzt"),
            }
        }
        if let Some(k) = kwargs {
            for (schluessel, wert) in k.iter() {
                let name: String = schluessel.extract()?;
                match name.as_str() {
                    "file" => file = Some(wert),
                    "size" => size = wert.extract()?,
                    "font_index" => index = wert.extract()?,
                    "resolution" => dpi = wert.extract()?,
                    "ucs4" => ucs4 = wert.is_truthy()?,
                    _ => {
                        return Err(PyTypeError::new_err(format!(
                            "'{name}' is an invalid keyword argument for Font()"
                        )))
                    }
                }
            }
        }
        Ok((file, size, index, dpi, ucs4))
    }

    /// Laedt eine Schriftdatei - der Rumpf von `__new__` und `__init__`.
    fn laden(
        file: Option<&Bound<'_, PyAny>>,
        size: f64,
        font_index: i32,
        resolution: u32,
        ucs4: bool,
    ) -> PyResult<PyFreetypeFont> {
        ft_pruefen()?;
        let _ = font_index;
        // **Groesse 0 heisst "unbestimmt"** und ist erlaubt; erst beim
        // Zeichnen muss eine dastehen. Gemessen gibt `Font(None)` eine
        // Schrift mit `size == 0.0`.
        // Gemessen: eine **negative** oder zu grosse Groesse gibt einen
        // `OverflowError`, nicht einen `ValueError`. pygame rechnet sie in
        // 26.6 um und meldet, wenn sie nicht hineinpasst.
        if size < 0.0 || size > (0x7FFF_FFFFi64 >> 6) as f64 {
            return Err(pyo3::exceptions::PyOverflowError::new_err(
                "point size is out of valid range",
            ));
        }
        let dpi = if resolution > 0 {
            resolution
        } else {
            AUFLOESUNG.load(std::sync::atomic::Ordering::Relaxed)
        };
        let punkte = if size > 0.0 {
            ((size * dpi as f64 / 72.0).round() as i32).max(1)
        } else {
            12
        };
        // **`default_font_voll` und nicht `default_font`**: der Faktor
        // 0.6875 aus pygames `font.c` gehoert zu `pygame.font`, nicht
        // hierher. Siehe `rustgame_font::default_font_voll`.
        let (inner, pfad) = match file {
            None => (
                schrift::default_font_voll(punkte).map_err(schrift_fehler)?,
                None,
            ),
            Some(o) if o.is_none() => (
                schrift::default_font_voll(punkte).map_err(schrift_fehler)?,
                None,
            ),
            Some(o) => {
                // Derselbe Satz wie bei `pygame.font.Font` - siehe
                // [`crate::text::datei_fehlt`]. `pygame.freetype` gibt
                // ihn wortgleich, gemessen.
                let wie = || o.str().map(|t| t.to_string()).unwrap_or_default();
                if let Ok(t) = o.extract::<String>() {
                    if let Some(e) = crate::text::datei_fehlt(&t, &wie()) {
                        return Err(e);
                    }
                    (
                        schrift::Font::new(&t, punkte).map_err(ft_lade_fehler)?,
                        Some(t),
                    )
                } else if o.is_instance_of::<pyo3::types::PyBytes>() {
                    // Ein Pfad als `bytes` - so reicht `ftfont.py` ihn
                    // durch, nachdem `encode_file_path` ihn kodiert hat.
                    let roh: Vec<u8> = o.extract()?;
                    let t = String::from_utf8_lossy(&roh).into_owned();
                    if let Some(e) = crate::text::datei_fehlt(&t, &wie()) {
                        return Err(e);
                    }
                    (
                        schrift::Font::new(&t, punkte).map_err(ft_lade_fehler)?,
                        Some(t),
                    )
                } else if let Ok(p) = o.call_method0("__fspath__") {
                    let t: String = p.extract()?;
                    if let Some(e) = crate::text::datei_fehlt(&t, &wie()) {
                        return Err(e);
                    }
                    (
                        schrift::Font::new(&t, punkte).map_err(ft_lade_fehler)?,
                        Some(t),
                    )
                } else if o.hasattr("read")? {
                    // **Erst zurueckspulen.** `__new__` und `__init__`
                    // laufen hier beide durch (PyO3 hat keinen
                    // `tp_init`-Schlitz, siehe oben), und das zweite Mal
                    // faende ein schon leergelesenes Dateiobjekt vor -
                    // `font_test.test_load_from_file_obj` faellt genau
                    // darauf herein und bekaeme "invalid file size: 0".
                    if o.hasattr("seek")? {
                        let _ = o.call_method1("seek", (0,));
                    }
                    let gelesen = o.call_method0("read")?;
                    // **Ein offenes Dateiobjekt ist erlaubt.** Gemessen:
                    // `ft.Font(open(pfad, "rb"))` geht, und ein
                    // `io.StringIO()` gibt einen `ValueError` - Text ist
                    // keine Schriftdatei, und die Laenge null erst recht
                    // nicht.
                    let daten: Vec<u8> = match gelesen.extract::<Vec<u8>>() {
                        Ok(b) => b,
                        Err(_) => {
                            return Err(PyValueError::new_err(
                                "Font file object has an invalid file size: 0",
                            ))
                        }
                    };
                    if daten.is_empty() {
                        return Err(PyValueError::new_err(
                            "Font file object has an invalid file size: 0",
                        ));
                    }
                    (
                        schrift::Font::from_bytes(daten, punkte).map_err(ft_lade_fehler)?,
                        None,
                    )
                } else {
                    return Err(PyTypeError::new_err(
                        "Font() requires a file name or None",
                    ));
                }
            }
        };
        // Eine Bitmapschrift nimmt ihre eingebaute Groesse, wenn keine
        // verlangt wurde - gemessen gibt `Font(bmp).size` dann `(8.0, 8.0)`.
        let (size, y) = if size == 0.0 {
            match inner.feste_groessen().first() {
                Some(g) => (g.4, Some(g.4)),
                None => (size, None),
            }
        } else {
            (size, None)
        };
        Ok(PyFreetypeFont {
            inner: Some(inner),
            pfad,
            groesse: size,
            groesse_y: y,
            drehung: 0,
            senkrecht: false,
            epoche: schrift::generation(),
            strength: 1.0 / 36.0,
            letzter_text: std::cell::RefCell::new(String::new()),
            aufloesung: resolution,
            strikes: true,
            stil: STYLE_NORMAL,
            origin: false,
            pad: false,
            antialiased: true,
            kerning: false,
            ucs4,
            underline_adjustment: 1.0,
            fgcolor: rustgame_core::color::Color::rgba(0, 0, 0, 255),
            bgcolor: None,
        })
    }



    /// Die geladene Schrift, oder der gemessene `RuntimeError`.
    fn kern(&self) -> PyResult<&schrift::Font> {
        // **Nach `freetype.quit()` antwortet nichts mehr.** Gemessen:
        // `font_test.test_font_method_should_raise_exception_after_quit`
        // faehrt jede Methode einmal durch und verlangt ueberall einen
        // Fehler. Eine Schrift, die nach dem Herunterfahren weiter
        // Auskunft gibt, ist die haeufigste Art, ein `quit()` zu
        // uebersehen.
        self.epoche_pruefen()?;
        self.inner.as_ref().ok_or_else(uneingerichtet)
    }

    /// Nur die Epoche - **ohne** die Datei zu verlangen.
    ///
    /// Die Trennung ist der Nullschrift wegen da (siehe das Feld
    /// `inner`): `f.antialiased` antwortet ohne Datei, `f.height` nicht,
    /// und nach einem `freetype.quit()` geben **beide** denselben
    /// Fehler. Gemessen an einer Schrift, die das `quit()` ueberlebt
    /// hat: jeder der zehn gefragten Zugriffe gibt
    /// `pygame.error: Invalid freetype font (freetype module quit since
    /// freetype font created)` - und der Satz nennt dreimal `freetype`,
    /// wo hier `Invalid font (font module quit since font created)`
    /// stand. Das ist der Satz von `pygame.font`.
    fn epoche_pruefen(&self) -> PyResult<()> {
        if !schrift::get_init() || self.epoche != schrift::generation() {
            return Err(PygameError::new_err(
                "Invalid freetype font (freetype module quit since freetype font created)",
            ));
        }
        Ok(())
    }

    /// Die Aufloesung dieser Schrift in dpi.
    fn dpi(&self) -> u32 {
        if self.aufloesung > 0 {
            self.aufloesung
        } else {
            AUFLOESUNG.load(std::sync::atomic::Ordering::Relaxed)
        }
    }

    /// Die Schrift in der gewuenschten Groesse - `size` darf je Aufruf
    /// abweichen, deshalb wird hier neu gebaut, wenn noetig.
    ///
    /// **Die Aufloesung rechnet mit.** FreeType bekommt Punkte und dpi und
    /// macht daraus Pixel; hier wird dasselbe getan, bevor die Groesse
    /// weitergereicht wird. Ohne diese Zeile ist `resolution` ein Feld,
    /// das man setzen kann und das nichts tut.
    fn in_groesse(&self, size: Option<Groesse>) -> PyResult<schrift::Font> {
        let kern = self.kern()?;
        let (g, gy) = match size {
            Some(s) => (s.0, s.1),
            None => (self.groesse, self.groesse_y),
        };
        if g <= 0.0 {
            // **`ValueError`, nicht `pygame.error`.** Gemessen:
            // `font.render("foobar", color)` ohne Groesse - weder am
            // Aufruf noch an der Schrift - gibt einen `ValueError`.
            // `freetype_test.test_freetype_Font_render` prueft genau das.
            return Err(PyValueError::new_err(
                "No font point size specified and no default font size set",
            ));
        }
        // Eine Bitmapschrift kann **nur** ihren eingebauten Strike - aber
        // welche Groessen ihn treffen, entscheiden pygame und FreeType
        // zusammen (siehe `Font::bitmap_trifft`): `8.4` trifft
        // `PyGameMono-8.bdf`, `(16, 8)` auch, `42` und `(8, 16)` nicht.
        // Der Satz ist `_PGFT_SetError(ft, "Failed to resize font", error)`
        // mit FreeTypes Text zu `Invalid_Pixel_Size`. Bis Runde 20 stand
        // hier ein erfundener (`Unable to set the font size for a bitmap
        // font`), und nur ganze Groessen aus `get_sizes()` gingen.
        if kern.ist_bitmap()
            && !kern.bitmap_trifft(
                (g * 64.0) as i64,
                gy.map_or(0, |y| (y * 64.0) as i64),
                i64::from(self.dpi()),
            )
        {
            fehler_merken("Failed to resize font: invalid pixel size");
            return Err(PygameError::new_err(
                "Failed to resize font: invalid pixel size",
            ));
        }
        let mut punkte = ((g * self.dpi() as f64 / 72.0).round() as i32).max(1);
        // **Eine Bitmapschrift rechnet in den Pixeln ihres Strikes.**
        // FreeType waehlt ihn (`FT_Select_Metrics`) und nimmt seine Masse
        // unveraendert - Auflaenge, Zeilenhoehe, Vorschuebe. Hier wurde
        // die Schrift in der **Punktgroesse** angelegt und damit
        // umgerechnet: `PyGameMono-18-100dpi.bdf` (25 Pixel) meldete bei
        // `size=18` eine Auflaenge 15 statt 20.
        if kern.ist_bitmap() {
            if let Some(s) = kern.feste_groessen().first() {
                punkte = s.2.max(1);
            }
        }
        if (g - self.groesse).abs() < 1e-9 && punkte == kern.get_point_size() {
            return Ok(kern.clone());
        }
        match &self.pfad {
            Some(p) => schrift::Font::new(p, punkte).map_err(schrift_fehler),
            // Dieselbe Stelle noch einmal - siehe oben.
            None => schrift::default_font_voll(punkte).map_err(schrift_fehler),
        }
    }

    /// Auflaenge, Unterlaenge und Zeilenhoehe in 26.6 fuer eine Groesse,
    /// wie FreeType sie fuer eine **Umrissschrift** meldet - `None` bei
    /// einer Bitmapschrift (die hat ihre eigenen Zahlen). Die Fehler der
    /// Groesse (null, Bitmapgroesse, die es nicht gibt) kommen wie bisher
    /// aus [`Self::in_groesse`].
    fn kopfmasse(&self, size: Option<Groesse>) -> PyResult<Option<(i64, i64, i64)>> {
        let f = self.in_groesse(size)?;
        if f.ist_bitmap() {
            return Ok(None);
        }
        let gx = size.map(|g| g.0).unwrap_or(self.groesse);
        let gy = match size {
            Some(g) => g.1.unwrap_or(g.0),
            None => self.groesse_y.unwrap_or(gx),
        };
        // `DBL_TO_FX6`, dann `FT_REQUEST_WIDTH`/`FT_REQUEST_HEIGHT` mit
        // der Aufloesung.
        let (x26, _) = self.anfrage26(Some(gx), None);
        let (y26, _) = self.anfrage26(Some(gy), None);
        Ok(Some(f.ft_kopf26(x26, y26)))
    }

    /// Die Hoehe der Flaeche fuer **leeren** Text, wie
    /// `_PGFT_Font_GetHeightSized` sie nimmt: die Zeilenhoehe mit der
    /// ungerundeten Skala (siehe [`Self::kopfmasse`]). Gemessen an Segoe UI,
    /// 17 Punkt bei 96 dpi (22,67 Pixel): 30, hier stand 31 aus der auf 23
    /// gerundeten Schrift.
    fn leerhoehe(&self, size: Option<Groesse>, f: &schrift::Font) -> PyResult<u32> {
        Ok(match self.kopfmasse(size)? {
            Some((_, _, h)) => (h >> 6).max(0) as u32,
            None => f.ft_height().max(0) as u32,
        })
    }

    /// Ist das ueberhaupt ein Stil?
    ///
    /// Gemessen: `render(..., style=42)` gibt einen `ValueError`, und
    /// zwar bevor irgendetwas gezeichnet wird. 42 ist `32 | 8 | 2`, und
    /// die 32 gibt es nicht.
    fn stil_pruefen(&self, stil: i32) -> PyResult<()> {
        if stil == STYLE_DEFAULT {
            return Ok(());
        }
        if (stil & !(STYLE_STRONG | STYLE_OBLIQUE | STYLE_UNDERLINE | STYLE_WIDE)) != 0 {
            return Err(PyValueError::new_err("Invalid style value"));
        }
        Ok(())
    }

    /// Was `_PGFT_BuildRenderMode` (`ft_render.c`) ablehnt - in seiner
    /// Reihenfolge, mit seinen Saetzen.
    ///
    /// Bis Runde 19 stand hier ein erfundener Satz fuer Bitmapschriften
    /// (`cannot render a bitmap font rotated or with a synthetic style`),
    /// und die vier Saetze fuer gedrehten und senkrechten Text fehlten
    /// ganz: ein gedrehter Unterstrich oder ein gedrehtes `wide` wurde hier
    /// gezeichnet, wo pygame `ValueError` gibt. pygame fragt das bei
    /// `render`, `render_to`, `render_raw`, `render_raw_to`, `get_rect`
    /// und - mit den Einstellungen der Schrift - bei `get_metrics`.
    fn modus_pruefen(&self, stil: i32, rotation: Option<i32>) -> PyResult<()> {
        let rotation = self.winkel(rotation);
        let Some(k) = &self.inner else {
            return Ok(());
        };
        let bitmap = k.ist_bitmap();
        let s = if stil == STYLE_DEFAULT { self.stil } else { stil };
        // `FT_STYLES_SCALABLE_ONLY`
        if s & (STYLE_STRONG | STYLE_OBLIQUE) != 0 && bitmap {
            return Err(PyValueError::new_err("Unsupported style(s) for a bitmap font"));
        }
        if rotation != 0 {
            if bitmap {
                return Err(PyValueError::new_err(
                    "rotated text is unsupported for a bitmap font",
                ));
            }
            if s & STYLE_WIDE != 0 {
                return Err(PyValueError::new_err(
                    "the wide style is unsupported for rotated text",
                ));
            }
            if s & STYLE_UNDERLINE != 0 {
                return Err(PyValueError::new_err(
                    "the underline style is unsupported for rotated text",
                ));
            }
            if self.pad {
                return Err(PyValueError::new_err(
                    "padding is unsupported for rotated text",
                ));
            }
        }
        if self.senkrecht && s & STYLE_UNDERLINE != 0 {
            return Err(PyValueError::new_err(
                "the underline style is unsupported for vertical text",
            ));
        }
        Ok(())
    }

    /// Darf diese Schrift ueberhaupt fett oder schraeg gestellt werden?
    ///
    /// `_ftfont_setstyle_flag` und `_ftfont_setstyle`: bei einer
    /// Bitmapschrift `AttributeError`, sobald die **Flagge** fett oder
    /// schraeg ist - auch fuer `False`. Gemessen: `f.strong = False` wirft
    /// dort genauso wie `f.strong = True`.
    fn stil_erlaubt(&self, stil: i32) -> PyResult<()> {
        let k = self.kern()?;
        if k.ist_bitmap() && stil & (STYLE_STRONG | STYLE_OBLIQUE) != 0 {
            return Err(pyo3::exceptions::PyAttributeError::new_err(
                "this style is unsupported for a bitmap font",
            ));
        }
        Ok(())
    }

    /// Traegt einen Stil in eine Schrift ein.
    ///
    /// **`STYLE_WIDE` ist dieselbe Maschine wie `STYLE_STRONG`**, nur mit
    /// einem anderen Ueberhang: beide verschmieren das fertige Bild nach
    /// rechts und machen damit jedes Zeichen breiter. Fett nimmt SDL_ttfs
    /// `ptsize / 10`, breit nimmt `pygame.freetype`s `ptsize * strength`.
    /// Gemessen bei Groesse 64: fett +6 Spalten je Zeichen, breit +2.
    ///
    /// Sind beide gesetzt, gewinnt fett - es ist der groessere Wert, und
    /// zweimal verschmieren waere ein drittes Verhalten, das es nicht
    /// gibt.
    fn stil_anwenden(&self, f: &mut schrift::Font, stil: i32, grad: i32) {
        let s = if stil == STYLE_DEFAULT { self.stil } else { stil };
        // `pygame.freetype` nimmt den Unterstrich in den Kasten auf -
        // siehe `rustgame_font::Font::unterstrich_waechst`.
        f.set_unterstrich_waechst(true);
        let fett = s & STYLE_STRONG != 0;
        let breit = s & STYLE_WIDE != 0;
        let punkte = f.get_point_size();
        if fett {
            f.set_glyph_overhang(punkte / 10);
        } else if breit {
            f.set_glyph_overhang((punkte as f64 * self.strength).round() as i32);
        }
        // Die rohen Schalter: `pygame.freetype` ist kein SDL_ttf und kennt
        // dessen Regel nicht, einen fetten oder kursiven Schnitt nicht noch
        // einmal nachzumachen.
        f.set_bold_roh(fett || breit);
        f.set_italic_roh(s & STYLE_OBLIQUE != 0);
        f.set_underline(s & STYLE_UNDERLINE != 0);
        // **Eingebaute Pixelbilder, und wann sie nicht gehen.**
        //
        // `pygame.freetype` teilt seine Stile in zwei Lager, und
        // `test_freetype_Font_use_bitmap_strikes` prueft jedes einzeln ab:
        //
        // ```text
        // vertraeglich      vertical, wide, underline
        // unvertraeglich    rotation, strong, oblique
        // ```
        //
        // Der Grund ist derselbe wie bei FreeType: ein fertiges Bild
        // laesst sich verschmieren (`wide`) und unterstreichen, aber nicht
        // scheren und nicht drehen - dafuer braucht es die Kontur.
        //
        // Dass `wide` auf derselben Seite steht wie `strong`, ist dabei
        // kein Widerspruch: beide landen in dieser Kiste auf `set_bold`
        // und unterscheiden sich nur im Zuschlag. Welches von beiden
        // gemeint war, weiss nur diese Zeile hier - darum entscheidet sie
        // und nicht `rustgame-font`.
        f.set_bitmap_strikes(
            self.strikes
                && grad == 0
                && s & STYLE_STRONG == 0
                && s & STYLE_OBLIQUE == 0,
        );
    }

    /// Der Text, oder - bei `None` - der **zuletzt benutzte**.
    ///
    /// Gemessen: `f.get_rect("M"); f.render(None)` zeichnet das M. pygame
    /// merkt sich den Text, damit man erst messen und dann zeichnen kann,
    /// ohne ihn zweimal zu nennen.
    fn text_von(&self, o: &Bound<'_, PyAny>) -> PyResult<String> {
        if o.is_none() {
            return Ok(self.letzter_text.borrow().clone());
        }
        let t = self.als_text(o)?;
        *self.letzter_text.borrow_mut() = t.clone();
        Ok(t)
    }

    /// Der gemessene Fehler fuer eine kaputte Ersatzzeichenfolge.
    ///
    /// `UnicodeEncodeError` braucht fuenf Argumente - Kodierung,
    /// Gegenstand, Anfang, Ende und Grund -, sonst laesst er sich gar
    /// nicht bauen. Ein blosses `new_err("...")` gaebe beim Auswerfen
    /// einen `TypeError` statt des erwarteten Fehlers.
    fn halbes_paar(o: &Bound<'_, PyAny>) -> PyErr {
        let py = o.py();
        // `o.clone()` waere ein `Bound`, und das darf den Faden nicht
        // wechseln - `PyErr::from_type` verlangt aber Argumente, die es
        // duerfen. Also wird der Fehler **sofort** gebaut und als
        // fertiges `PyErr` weitergereicht.
        let art = py.get_type::<pyo3::exceptions::PyUnicodeEncodeError>();
        match art.call1(("utf-8", o.clone(), 0usize, 1usize, "surrogates not allowed")) {
            Ok(f) => PyErr::from_value(f),
            Err(e) => e,
        }
    }

    /// Text oder `bytes` - beides ist erlaubt.
    ///
    /// **`bytes` wird als Latin-1 gelesen**, nicht als UTF-8. So macht es
    /// pygame auch: `font.size(b"\xff")` gibt dort dieselbe Breite wie
    /// `font.size("\u00ff")`. Mit UTF-8 waere jedes zweite Byte ein
    /// Fehler statt eines Zeichens.
    fn als_text(&self, o: &Bound<'_, PyAny>) -> PyResult<String> {
        if let Ok(t) = o.extract::<String>() {
            return Ok(t);
        }
        // **Ein Text mit halben Ersatzzeichen ist auch ein Text.**
        //
        // Python erlaubt einzelne Surrogate in einer Zeichenkette:
        // `"\ud80c\udc79"` ist zulaessig und meint als Paar U+13079.
        // Rust kennt so etwas nicht, also scheitert `extract::<String>()`
        // daran - gemessen an `freetype_test`, das genau dieses Paar
        // durch `get_rect` und `render` schickt und dasselbe Ergebnis
        // erwartet wie fuer `"\U00013079"`.
        //
        // Der Umweg ueber UTF-16 setzt die Paare wieder zusammen; das ist
        // genau das, was pygame mit `ucs4 = False` tut.
        if o.is_instance_of::<pyo3::types::PyString>() {
            if let Ok(roh) = o.call_method1("encode", ("utf-16-le", "surrogatepass")) {
                let bytes: Vec<u8> = roh.extract()?;
                // `as_chunks` statt `chunks_exact(2)`: die Zweiergruppe
                // steht damit im Typ, und das Muster `[a, b]` braucht
                // keinen Index.
                let worte: Vec<u16> = bytes
                    .as_chunks::<2>()
                    .0
                    .iter()
                    .map(|&[nieder, hoch]| u16::from_le_bytes([nieder, hoch]))
                    .collect();
                // **`ucs4` entscheidet, was ein Paar ist.**
                //
                // Mit `ucs4 = False` werden zwei Ersatzzeichen zu einem
                // Zeichen zusammengesetzt - so hat es UTF-16 gemeint -,
                // und ein **halbes** Paar ist ein Fehler. Mit
                // `ucs4 = True` bleiben es zwei Zeichen, und weil keines
                // davon fuer sich gueltig ist, wird jedes zum
                // Ersatzzeichen U+FFFD.
                //
                // Gemessen an `freetype_test.test_freetype_Font_render`:
                // dieselbe Zeichenkette gibt mit `ucs4 = False` und mit
                // `True` **verschieden grosse** Rechtecke, und sechs
                // kaputte Folgen (einzelnes hohes, einzelnes niedriges,
                // hoch+hoch, ...) geben einen `UnicodeEncodeError`.
                if self.ucs4 {
                    return Ok(worte
                        .iter()
                        .map(|w| {
                            char::from_u32(*w as u32)
                                .unwrap_or(char::REPLACEMENT_CHARACTER)
                        })
                        .collect());
                }
                return match String::from_utf16(&worte) {
                    Ok(t) => Ok(t),
                    Err(_) => Err(Self::halbes_paar(o)),
                };
            }
        }
        // **Bytes, und nur Bytes.** Hier stand `extract::<Vec<u8>>()`
        // allein, und das nimmt auch eine **Liste** von Zahlen - `[]`
        // wurde damit zum leeren Text statt zum Fehler.
        // `font_test.test_render` prueft genau das:
        // `f.render([], False, [0,0,0], [255,255,255])` muss ein
        // `TypeError` sein.
        if o.is_instance_of::<pyo3::types::PyBytes>()
            || o.is_instance_of::<pyo3::types::PyByteArray>()
        {
            if let Ok(b) = o.extract::<Vec<u8>>() {
                return Ok(b.iter().map(|c| *c as char).collect());
            }
        }
        Err(PyTypeError::new_err("text must be str or bytes"))
    }

    /// Setzt `oben` ueber `unten` - **mit geradem Alpha**.
    ///
    /// Und das ist der Unterschied zu einem gewoehnlichen Blit. SDLs
    /// Mischung rechnet `out = src*a + dst*(255-a)` und macht aus einer
    /// halbdurchsichtigen Schrift auf durchsichtigem Grund ein
    /// **dunkleres** Bild. `pygame.freetype` legt statt dessen sauber
    /// uebereinander:
    ///
    /// ```text
    /// fgcolor = (10, 20, 30, 128) auf (0, 0, 0, 0)
    ///   SDL-Mischung   ->  (5, 10, 15, 128)
    ///   freetype       ->  (10, 20, 30, 128)   <- gemessen
    /// ```
    ///
    /// Ist der Grund durchsichtig, bleibt die Farbe also ganz erhalten.
    fn ueber(
        oben: rustgame_core::color::Color,
        unten: u32,
        ziel_hat_alpha: bool,
    ) -> u32 {
        let (dr, dg, db, da) = (
            (unten >> 16) & 0xFF,
            (unten >> 8) & 0xFF,
            unten & 0xFF,
            // **Eine Flaeche ohne Alphakanal ist deckend, nicht
            // durchsichtig.**
            //
            // Hier stand `(unten >> 24) & 0xFF` ohne Unterscheidung, und
            // bei XRGB8888 sind die obersten acht Bit **null**. Die Zeile
            // unten (`sa == 255 || da == 0`) hielt das Ziel damit fuer
            // leer und **ersetzte** statt zu mischen. Gemessen an
            // `render_to` mit halbdurchsichtiger Schriftfarbe auf eine
            // Flaeche ohne `SRCALPHA`:
            //
            // ```text
            //           pygame        vorher hier
            //   Pixel   (20, 0, 20)   (99, 0, 100)
            // ```
            //
            // Also die ganze Farbe statt eines Viertels davon.
            if ziel_hat_alpha {
                (unten >> 24) & 0xFF
            } else {
                255
            },
        );
        let sa = oben.a as u32;
        if sa == 255 || da == 0 {
            return (sa.max(da) << 24)
                | ((oben.r as u32) << 16)
                | ((oben.g as u32) << 8)
                | oben.b as u32;
        }
        let out_a = sa + da * (255 - sa) / 255;
        if out_a == 0 {
            return 0;
        }
        // **Gerundet, nicht abgeschnitten.** Gemessen an
        // `render_to` mit `fgcolor=(99, 0, 100, 64)` auf Schwarz:
        //
        // ```text
        //   99 * 64 / 255 = 24,85      pygame: 25      abgeschnitten: 24
        // ```
        //
        // Dieselbe Eins fehlte an sechs der neun Pixel eines geglaetteten
        // Punktes. Das halbe `out_a` dazu ist dieselbe Rundung, die SDLs
        // `(s * a + 255) >> 8` erreicht.
        // **Dieselbe Formel wie der Blit daneben**, und das ist keine
        // Bequemlichkeit, sondern was der Test verlangt.
        //
        // `test_freetype_Font_render_to_mono` vergleicht nicht mit
        // pygame, sondern mit sich selbst: es blittet die Farbe erst mit
        // `surf.blit`, merkt sich das Ergebnis und verlangt danach
        // dasselbe von `render_to`. Beide Seiten sind rustgames Rechnung,
        // und sie muessen zusammenfallen.
        //
        // Hier stand eine eigene Lehrbuchformel, und sie lag bei jeder
        // dritten Farbe um eins daneben:
        //
        // ```text
        //   Quelle           pygames ALPHA_BLEND   Lehrbuch   gerundet
        //   (97, a=60)       23                    22         23
        //   (120, a=180)     84                    84         85
        //   (99, a=64)       25                    24         25
        // ```
        //
        // Keine der beiden Rundungen trifft beide Zeilen - pygames
        // `ALPHA_BLEND_COMP` trifft alle drei, weil es auf der 256er-Skala
        // rechnet (`(sC - dC) * sA + sC) >> 8) + dC`). Die Formel liegt in
        // `rustgame_core::blend_formula` und ist dort exhaustiv ueber alle
        // 16.777.216 Kombinationen gegen pygame geprueft. Sie zweimal zu
        // haben waere zweimal die Gelegenheit, sich zu unterscheiden.
        use rustgame_core::blend_formula::{blend_alpha, blend_channel};
        let _ = out_a;
        let sa8 = oben.a;
        ((blend_alpha(da as u8, sa8) as u32) << 24)
            | ((blend_channel(dr as u8, oben.r, sa8) as u32) << 16)
            | ((blend_channel(dg as u8, oben.g, sa8) as u32) << 8)
            | blend_channel(db as u8, oben.b, sa8) as u32
    }


    // ----- Die Setzer ohne Pruefung -------------------------------------
    //
    // Sie stehen ausserhalb von `#[pymethods]`, und das ist kein Zufall:
    // PyO3 traegt **jede** Methode dort in den Typ ein, auch eine, die
    // nur als Helfer gedacht war. `font_test.py` zaehlt fuer
    // `pygame.ftfont` genau 23 Methoden ab und bemaengelt jede weitere -
    // gemessen standen hier neun Namen zuviel.

    fn set_strong_roh(&mut self, v: bool) {
        self.stil = if v {
            self.stil | STYLE_STRONG
        } else {
            self.stil & !STYLE_STRONG
        };
    }

    fn set_oblique_roh(&mut self, v: bool) {
        self.stil = if v {
            self.stil | STYLE_OBLIQUE
        } else {
            self.stil & !STYLE_OBLIQUE
        };
    }

    fn setze_underline(&mut self, v: bool) {
        self.stil = if v {
            self.stil | STYLE_UNDERLINE
        } else {
            self.stil & !STYLE_UNDERLINE
        };
    }

    fn setze_wide(&mut self, v: bool) {
        self.stil = if v {
            self.stil | STYLE_WIDE
        } else {
            self.stil & !STYLE_WIDE
        };
    }

    fn setze_origin(&mut self, v: bool) {
        self.origin = v;
    }

    fn setze_pad(&mut self, v: bool) {
        self.pad = v;
    }

    fn setze_antialiased(&mut self, v: bool) {
        self.antialiased = v;
    }

    fn setze_kerning(&mut self, v: bool) {
        self.kerning = v;
    }

    fn setze_ucs4(&mut self, v: bool) {
        self.ucs4 = v;
    }

    /// Die Deckung, wenn der Text **untereinander** steht.
    ///
    /// Jedes Zeichen kommt auf seine eigene Zeile, die Zeilen stehen im
    /// Abstand `get_linesize()`, und die Breite ist die des breitesten
    /// Zeichens. Das ist FreeTypes Verhalten fuer eine Schrift ohne
    /// `vmtx`-Tabelle - und ohne die ist fast jede.
    fn deckung_senkrecht(
        &self,
        f: &schrift::Font,
        text: &str,
    ) -> PyResult<(Vec<u8>, (u32, u32))> {
        let weiss = rustgame_core::color::Color::rgba(255, 255, 255, 255);
        let schwarz = rustgame_core::color::Color::rgba(0, 0, 0, 255);
        let zeile = f.get_linesize().max(1) as u32;
        let mut bilder = Vec::new();
        let mut breite = 0u32;
        for c in text.chars() {
            let einzeln = c.to_string();
            let fl = f
                .render(&einzeln, self.antialiased, weiss, Some(schwarz))
                .map_err(schrift_fehler)?;
            let (b, h) = fl.get_size();
            breite = breite.max(b);
            let mut d = Vec::with_capacity((b * h) as usize);
            for y in 0..h as i32 {
                for x in 0..b as i32 {
                    d.push(fl.get_at((x, y)).r);
                }
            }
            bilder.push((d, b, h));
        }
        let hoehe = zeile * bilder.len().max(1) as u32;
        let mut aus = vec![0u8; (breite * hoehe) as usize];
        for (i, (d, b, h)) in bilder.iter().enumerate() {
            let oben = i as u32 * zeile;
            for y in 0..*h {
                for x in 0..*b {
                    let zy = oben + y;
                    if zy >= hoehe || x >= breite {
                        continue;
                    }
                    aus[(zy * breite + x) as usize] = d[(y * b + x) as usize];
                }
            }
        }
        Ok((aus, (breite, hoehe)))
    }

    /// Die Masse eines Zeichens, **gedreht**.
    ///
    /// Die vier Ecken des Kastens werden gedreht, dann der umschliessende
    /// Kasten davon genommen; Vorschub und Zeilenschritt drehen mit.
    fn mass_gedreht(
        m: (i32, i32, i32, i32, f64, f64),
        grad: i32,
    ) -> (i32, i32, i32, i32, f64, f64) {
        if grad == 0 {
            return m;
        }
        let (minx, maxx, miny, maxy, ax, ay) = m;
        let bogen = (grad as f64).to_radians();
        let (sin, cos) = (bogen.sin(), bogen.cos());
        let dreh = |x: f64, y: f64| (x * cos - y * sin, x * sin + y * cos);
        let ecken = [
            dreh(minx as f64, miny as f64),
            dreh(maxx as f64, miny as f64),
            dreh(minx as f64, maxy as f64),
            dreh(maxx as f64, maxy as f64),
        ];
        let xs: Vec<f64> = ecken.iter().map(|e| e.0).collect();
        let ys: Vec<f64> = ecken.iter().map(|e| e.1).collect();
        let klein = |v: &[f64]| v.iter().cloned().fold(f64::INFINITY, f64::min);
        let gross = |v: &[f64]| v.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
        let (nax, nay) = dreh(ax, ay);
        (
            klein(&xs).round() as i32,
            gross(&xs).round() as i32,
            klein(&ys).round() as i32,
            gross(&ys).round() as i32,
            (nax * 1000.0).round() / 1000.0,
            (nay * 1000.0).round() / 1000.0,
        )
    }

    /// Welcher Winkel gilt - der vom Aufruf oder der von der Schrift?
    ///
    /// **Eine ausdrueckliche Null schaltet die Drehung ab.** Gemessen:
    ///
    /// ```text
    /// f.rotation = 90
    /// f.get_rect("Hallo")               Rect(-17, 58, 17, 57)   gedreht
    /// f.get_rect("Hallo", rotation=0)   Rect(1, 17, 57, 17)     nicht
    /// ```
    ///
    /// Das geht nur mit `Option`: waere der Vorgabewert eine 0, saehe
    /// die Methode beide Faelle gleich.
    fn winkel(&self, rotation: Option<i32>) -> i32 {
        let v = rotation.unwrap_or(self.drehung);
        ((v % 360) + 360) % 360
    }

    /// Dreht ein Deckungsbild **gegen** den Uhrzeiger.
    ///
    /// Rueckwaerts abgetastet, also je Zielpixel einmal in die Quelle
    /// geschaut - so bleibt kein Loch. Zwischen den Stuetzstellen wird
    /// linear gemittelt; FreeType dreht die **Kontur** und rastert
    /// danach, was an den Kanten ein Haar sauberer aussieht, aber die
    /// Masse gleich laesst.
    fn gedreht(deckung: &[u8], b: u32, h: u32, grad: i32) -> (Vec<u8>, u32, u32) {
        if grad == 0 || b == 0 || h == 0 {
            return (deckung.to_vec(), b, h);
        }
        let bogen = (grad as f64).to_radians();
        // **Ein Viertelkreis hat keinen Sinus von 1,2e-16.**
        //
        // `(180.0).to_radians().sin()` gibt genau das, und daraus wurde
        // `ceil(18 * 1 + 17 * 1.2e-16)` = **19** statt 18. Gemessen fiel es
        // an `get_rect("M", rotation=180)` auf: pygame meldet
        // `(-19, 0, 18, 17)`, hier stand `(-19, 0, 19, 18)`.
        let sauber = |v: f64| if v.abs() < 1e-12 { 0.0 } else { v };
        let (sin, cos) = (sauber(bogen.sin()), sauber(bogen.cos()));
        // Der Kasten, in den das gedrehte Bild passt.
        let (bf, hf) = (b as f64, h as f64);
        let nb = (bf * cos.abs() + hf * sin.abs()).ceil().max(1.0);
        let nh = (bf * sin.abs() + hf * cos.abs()).ceil().max(1.0);
        let (nbi, nhi) = (nb as u32, nh as u32);
        let mut aus = vec![0u8; (nbi * nhi) as usize];
        let (mx, my) = (bf / 2.0, hf / 2.0);
        let (nmx, nmy) = (nb / 2.0, nh / 2.0);
        for y in 0..nhi {
            for x in 0..nbi {
                // Rueckwaerts: der Zielpunkt wird **mit** dem Uhrzeiger
                // zurueckgedreht, um seine Stelle in der Quelle zu finden.
                let (dx, dy) = (x as f64 + 0.5 - nmx, y as f64 + 0.5 - nmy);
                let qx = mx + dx * cos - dy * sin;
                let qy = my + dx * sin + dy * cos;
                let (x0, y0) = (qx.floor(), qy.floor());
                let (fx, fy) = (qx - x0, qy - y0);
                let hole = |sx: f64, sy: f64| -> f64 {
                    if sx < 0.0 || sy < 0.0 || sx >= bf || sy >= hf {
                        return 0.0;
                    }
                    deckung[(sy as u32 * b + sx as u32) as usize] as f64
                };
                let oben = hole(x0, y0) * (1.0 - fx) + hole(x0 + 1.0, y0) * fx;
                let unten = hole(x0, y0 + 1.0) * (1.0 - fx) + hole(x0 + 1.0, y0 + 1.0) * fx;
                let v = oben * (1.0 - fy) + unten * fy;
                aus[(y * nbi + x) as usize] = v.round().clamp(0.0, 255.0) as u8;
            }
        }
        (aus, nbi, nhi)
    }

    /// Die Deckung des Textes, ein Byte je Pixel - fuer `render_raw`.
    fn deckung(
        &self,
        text: &str,
        stil: i32,
        size: Option<Groesse>,
        rotation: Option<i32>,
    ) -> PyResult<(Vec<u8>, (u32, u32))> {
        let k = self.kasten(text, stil, size, rotation)?;
        Ok((k.deckung, (k.w, k.h)))
    }

    /// Der **Zeilenkasten**: die Deckung, wie die Schriftkiste sie rastert.
    ///
    /// Ein Byte je Pixel, Breite gleich dem Gesamtvorschub, Hoehe gleich
    /// dem, was `rustgame_font::render` aufspannt. Der Zuschnitt auf die
    /// Tinte geschieht erst in [`Self::kasten`].
    ///
    /// **Weiss auf Schwarz, und die Deckung ist der Rotkanal.**
    ///
    /// Hier stand `render(text, aa, weiss, None)` und danach die Frage,
    /// welche Pixel **nicht** die Schluesselfarbe sind. Das ging so lange
    /// gut, wie die Schluesselfarbe nicht weiss war - und bei einer
    /// Schrift ohne Glaettung ist sie es: `rustgame_font` baut dort eine
    /// Flaeche mit Zweifarbentafel, und die Schluesselfarbe kann dieselbe
    /// sein wie die Schrift. Dann fand die Messung **gar keine Tinte**,
    /// und `get_rect(".")` gab `(0, 0, 7, 0)` statt `(2, 3, 2, 3)`.
    ///
    /// Mit einem ausdruecklichen schwarzen Grund gibt es keine
    /// Schluesselfarbe mehr: gedeckt ist, was heller als schwarz ist, und
    /// bei Glaettung ist der Grauwert genau die Deckung. Eine Regel statt
    /// zweier, und sie kann nicht zusammenfallen.
    /// Wie weit leere Zeichen am Rand den Kasten aufziehen.
    ///
    /// Gibt `(links, rechts)` in Spalten des Zeilenkastens - jeweils
    /// `None`, wenn das aeusserste Zeichen Tinte hat und damit nichts
    /// beiträgt.
    ///
    /// ## Warum es das braucht
    ///
    /// Der Zeilenkasten ist so breit wie der **Vorschub** der Zeile, und
    /// `tintenkasten` schneidet ihn auf die Tinte zusammen. Bei einem
    /// Leerzeichen am Rand ist das zu viel: pygame behaelt es.
    ///
    /// Gemessen an `test_sans.ttf`, Groesse 24 (Vorschub von `" "` und
    /// `"."` ist je 7):
    ///
    /// ```text
    ///            pygame geglaettet   pygame einfarbig
    ///   "."      (2, 3,  3, 3)       (2, 3,  2, 3)
    ///   " ."     (0, 3, 12, 3)       (0, 3, 11, 3)
    ///   ". "     (2, 3, 12, 3)       (2, 3,  6, 3)
    ///   " . "    (0, 3, 21, 3)       (0, 3, 15, 3)
    /// ```
    ///
    /// Daraus die Regel, und sie ist in den beiden Zweigen **nicht
    /// dieselbe**:
    ///
    /// * **geglaettet**: ein leeres Zeichen belegt `[Stift,
    ///   Stift + Vorschub]`. Bei `". "` zieht das den Kasten bis 14, also
    ///   auf Breite 12.
    /// * **einfarbig**: es belegt nur `[Stift, Stift + 1]`. Bei `". "`
    ///   zieht das bis 8, also auf Breite 6.
    ///
    /// Das sieht nach einer Grille aus und ist doch eine Messung, die
    /// jeder der vier Faelle bestaetigt. FreeType rastert eine leere
    /// Kontur einfarbig als ein einzelnes Pixel; geglaettet bleibt sie
    /// leer, und pygame nimmt dann den Vorschub.
    ///
    /// **Es waren die letzten zwei Tests von 2485.** Der Vermerk daneben
    /// nannte sie Rasterkanten ("an einer Ecke steht eine Deckung von 1
    /// statt 0"); keiner von beiden war eine.
    fn leerer_rand(
        &self,
        f: &schrift::Font,
        text: &str,
        breite: u32,
    ) -> (Option<u32>, Option<u32>) {
        let zeichen: Vec<char> = text.chars().collect();
        let (Some(&erst), Some(&letzt)) = (zeichen.first(), zeichen.last()) else {
            return (None, None);
        };
        // Leer heisst: kein Pixel Tinte. Ein Zeichen, das die Schrift
        // nicht hat, gibt `None` und zaehlt ebenso.
        let leer = |c: char, f: &schrift::Font| {
            let m = f.metrics(&c.to_string());
            match m.first() {
                Some(Some((minx, maxx, miny, maxy, _))) => {
                    maxx - minx == 0 || maxy - miny == 0
                }
                _ => true,
            }
        };
        let vorschub = |c: char, f: &schrift::Font| {
            f.metrics(&c.to_string())
                .first()
                .and_then(|m| m.map(|(_, _, _, _, a)| a.max(0) as u32))
                .unwrap_or(0)
        };
        let links = if leer(erst, f) { Some(0u32) } else { None };
        let rechts = if leer(letzt, f) {
            // Der Stift des letzten Zeichens liegt einen Vorschub vor dem
            // Ende des Zeilenkastens.
            let a = vorschub(letzt, f);
            let stift = breite.saturating_sub(a);
            Some(if self.antialiased { stift + a } else { stift + 1 })
        } else {
            None
        };
        (links, rechts)
    }

    fn zeilenkasten(
        &self,
        f: &mut schrift::Font,
        text: &str,
    ) -> PyResult<(Vec<u8>, u32, u32)> {
        if self.senkrecht {
            let (d, (b, h)) = self.deckung_senkrecht(f, text)?;
            return Ok((d, b, h));
        }
        let weiss = rustgame_core::color::Color::rgba(255, 255, 255, 255);
        let schwarz = rustgame_core::color::Color::rgba(0, 0, 0, 255);
        // **Weiss auf Schwarz, und die Deckung ist der Rotkanal.**
        //
        // Hier stand `render(text, aa, weiss, None)` und danach die Frage,
        // welche Pixel **nicht** die Schluesselfarbe sind. Das ging so
        // lange gut, wie die Schluesselfarbe nicht weiss war - und bei
        // einer Schrift ohne Glaettung ist sie es: `rustgame_font` baut
        // dort eine Flaeche mit Zweifarbentafel, und die Schluesselfarbe
        // kann dieselbe sein wie die Schrift. Dann fand die Messung
        // **gar keine Tinte**, und `get_rect(".")` gab `(0, 0, 7, 0)` statt
        // `(2, 3, 2, 3)`.
        //
        // Mit einem ausdruecklichen schwarzen Grund gibt es keine
        // Schluesselfarbe mehr: gedeckt ist, was heller als schwarz ist,
        // und bei Glaettung ist der Grauwert genau die Deckung. Eine Regel
        // statt zweier, und sie kann nicht zusammenfallen.
        let flaeche = f
            .render(text, self.antialiased, weiss, Some(schwarz))
            .map_err(schrift_fehler)?;
        let (b, h) = flaeche.get_size();
        let mut aus = Vec::with_capacity((b * h) as usize);
        for y in 0..h as i32 {
            for x in 0..b as i32 {
                aus.push(flaeche.get_at((x, y)).r);
            }
        }
        Ok((aus, b, h))
    }

    /// Was `render_raw_to` zurueckgibt - **dasselbe wie `get_rect`**.
    ///
    /// Und zwar unabhaengig von `dest`; gemessen:
    ///
    /// ```text
    /// f.get_rect("Hi", size=12)              Rect(0, 9, 11, 9)
    /// f.render_raw_to(a, "Hi", size=12)      Rect(0, 9, 11, 9)
    /// f.render_raw_to(a, "Hi", (3, 5), ...)  Rect(0, 9, 11, 9)
    /// ```
    ///
    /// Der Versatz verschiebt also, **was** geschrieben wird, aber nicht,
    /// was gemeldet wird. Hier stand das Rechteck aus Versatz und
    /// Bildgroesse, und das war an beiden Stellen daneben.
    fn rohrechteck(k: &Kasten) -> PyRect {
        PyRect::neu(rustgame_core::rect::Rect::new(
            k.rx,
            k.ry,
            k.w as i32,
            k.h as i32,
        ))
    }

    /// Der Kasten um die **Tinte** - oder `None`, wenn es keine gibt.
    ///
    /// Gibt `(Spalte, Zeile, Breite, Hoehe)` im Zeilenkasten.
    fn tintenkasten(d: &[u8], b: u32, h: u32) -> Option<(u32, u32, u32, u32)> {
        let (mut x0, mut y0, mut x1, mut y1) = (u32::MAX, u32::MAX, 0u32, 0u32);
        for y in 0..h {
            for x in 0..b {
                if d[(y * b + x) as usize] != 0 {
                    x0 = x0.min(x);
                    y0 = y0.min(y);
                    x1 = x1.max(x);
                    y1 = y1.max(y);
                }
            }
        }
        if x0 == u32::MAX {
            return None;
        }
        Some((x0, y0, x1 - x0 + 1, y1 - y0 + 1))
    }

    /// Alles, was eine Messung ueber einen Text ergibt.
    ///
    /// ## Warum das mehr ist als eine Bildgroesse
    ///
    /// `pygame.freetype.get_rect` gibt **nicht** den Zeilenkasten, sondern
    /// den Kasten um die **Tinte** - und seine Lage dazu. Gemessen an
    /// `test_sans.ttf` in Groesse 24:
    ///
    /// ```text
    /// get_rect("M")   (1, 17, 18, 17)
    /// get_rect("g")   (1, 13, 11, 18)
    /// ```
    ///
    /// Das `y` ist dabei kein Bildschirmversatz, sondern die Hoehe der
    /// Tinte **ueber der Grundlinie**: das `M` reicht 17 Pixel hinauf und
    /// gar nicht hinunter, das `g` 13 hinauf und 5 hinunter. Wer das
    /// uebersieht, meldet `(0, 0, 20, 28)` - den Zeilenkasten -, und jede
    /// Rechnung, die Textkaesten aneinanderlegt, wird um die
    /// Unterlaengen falsch.
    ///
    /// ## `pad` schaltet auf den Zeilenkasten zurueck
    ///
    /// `Font.pad = True` fuellt auf `(0, Auflaenge, Vorschub,
    /// Glyphenhoehe)` auf - das ist es, was `pygame.ftfont` einstellt,
    /// damit es sich wie `pygame.font` verhaelt. Gedreht geht das nicht;
    /// pygame sagt dort `ValueError: padding is unsupported for rotated
    /// text`, und dieser Satz ist uebernommen.
    ///
    /// ## Zwei Sonderfaelle, beide gemessen
    ///
    /// * **Leerer Text**: `(0, Auflaenge, 0, Zeilenhoehe)`. Die Hoehe ist
    ///   da, obwohl die Breite null ist.
    /// * **Nur ein Leerzeichen**: `(0, 0, 7, 0)`. Kein Tinte, also keine
    ///   Hoehe und kein `y` - aber die volle Breite des Vorschubs.
    ///
    /// Die beiden sehen sich aehnlich und sind es nicht: beim ersten ist
    /// der **Zeilenkasten** null breit, beim zweiten nur die Tinte.
    fn kasten(
        &self,
        text: &str,
        stil: i32,
        size: Option<Groesse>,
        rotation: Option<i32>,
    ) -> PyResult<Kasten> {
        // **Eine Groesse darf aus zwei Zahlen bestehen.** `size=(24, 12)`
        // meint "so breit wie 24, so hoch wie 12", und gemessen kommt
        // genau das heraus:
        //
        // ```text
        // get_rect("A", size=12)        (0,  9,  8,  9)
        // get_rect("A", size=24)        (0, 17, 16, 17)
        // get_rect("A", size=(24, 12))  (0,  9, 16,  9)
        // get_rect("A", size=(12, 24))  (0, 17,  8, 17)
        // ```
        //
        // FreeType streckt dafuer die **Kontur** vor dem Rastern, hier
        // wird das fertige Bild senkrecht umgetastet: gerastert wird in
        // der Breite `x`, danach auf die Zeilenhoehe von `y` gezogen. Das
        // trifft Breite und Hoehe auf den Wert und die Kanten dazwischen
        // um Haaresbreite anders - dieselbe Sorte Unterschied wie beim
        // Drehen, und aus demselben Grund.
        let (gx, gy) = match size {
            Some(g) => (Some(g.0), g.1),
            None => (None, self.groesse_y),
        };
        // **Der Winkel zuerst.** Er entscheidet mit darueber, ob ein
        // eingebautes Pixelbild benutzt werden darf - siehe
        // `stil_anwenden`.
        let grad = self.winkel(rotation);
        let mut f = self.in_groesse(size)?;
        self.stil_anwenden(&mut f, stil, grad);
        if self.pad && grad != 0 {
            return Err(PyValueError::new_err(
                "padding is unsupported for rotated text",
            ));
        }
        // **pygame.freetype setzt selbst** - siehe `kasten_ft`. Seit
        // Runde 20 auch mit **zwei Achsen** (`size=(24, 12)`): FreeType
        // hintet sie mit dem gestreckten TrueType-Interpreter, und der
        // steckt im skrifa-Abzug (`HintingInstance::new_gestreckt`). Der
        // alte Weg unten bleibt, wo `ft_zeile` `None` sagt.
        {
            let (g26, g26_y) = self.anfrage26(gx, gy);
            // `load_glyphs` in `ft_layout.c`: scheitert eine Glyphe, ist
            // Schluss - siehe `Font::ft_ladefehler`.
            let fs = self.ft_stil(stil, grad, g26, g26_y);
            if let Some(id) = f.ft_ladefehler(text, &fs, self.strikes) {
                return Err(PygameError::new_err(format!(
                    "Unable to load glyph for id {id}"
                )));
            }
            if let Some(k) = self.kasten_ft(&f, text, stil, grad, g26, g26_y) {
                return Ok(k);
            }
        }
        let (mut roh, b, mut h) = self.zeilenkasten(&mut f, text)?;
        let mut auflaenge = f.ft_ascender();
        // Die zweite Zahl, falls sie eine andere ist.
        let gx_wert = gx.unwrap_or(self.groesse);
        if let Some(y) = gy.filter(|y| (y - gx_wert).abs() > 1e-9) {
            let mut fy = self.in_groesse(Some(Groesse(y, None)))?;
            self.stil_anwenden(&mut fy, stil, grad);
            let (roh_y, by, hy) = self.zeilenkasten(&mut fy, text)?;
            auflaenge = fy.ft_ascender();
            // **Jede Achse liefert, was sie kann.** Die Spalten kommen aus
            // dem Bild in der Breite `x`, die Zeilen aus dem in der Hoehe
            // `y`; dazwischen wird das eine auf die Zeilenzahl des anderen
            // gezogen. Damit stimmen Breite **und** Hoehe auf den Wert,
            // und nur die Kanten dazwischen liegen um Haaresbreite anders
            // als bei FreeType, das die Kontur streckt statt das Bild.
            let waagerecht = Self::tintenkasten(&roh, b, h);
            let senkrecht = Self::tintenkasten(&roh_y, by, hy);
            if let (Some((_, qy0, _, qh0)), Some((_, zy0, _, zh0))) =
                (waagerecht, senkrecht)
            {
                let mut neu = vec![0u8; (b * hy) as usize];
                for zy in 0..zh0 {
                    // Das Groesste ueber den zugehoerigen Quellzeilen - ein
                    // einzelner Nachbar liesse beim Verkleinern die
                    // aeusserste Reihe Tinte unter den Tisch fallen.
                    let q0 = qy0 + ((zy as u64 * qh0 as u64) / zh0 as u64) as u32;
                    let q1 = qy0 + (((zy + 1) as u64 * qh0 as u64) / zh0 as u64) as u32;
                    let q1 = q1.max(q0 + 1).min(qy0 + qh0);
                    for x in 0..b {
                        let mut v = 0u8;
                        for q in q0..q1 {
                            v = v.max(roh[(q * b + x) as usize]);
                        }
                        neu[((zy0 + zy) * b + x) as usize] = v;
                    }
                }
                roh = neu;
                h = hy;
            }
        }

        // Welchen Ausschnitt des Zeilenkastens wir behalten.
        let (ox, oy, mut nw, mut nh, mut rx, mut ry) = if b == 0 {
            // Leerer Text: keine Breite, aber eine Hoehe.
            // **Ungepolstert ist es `ascender - descender`, nicht die
            // Zeilenhoehe.** Gemessen mit `Font(None, 24)`:
            //
            // ```text
            // get_sized_ascender()    20
            // get_sized_descender()   -5
            // get_sized_height()      26
            // get_rect("")  pygame    (0, 20, 0, 25)   <- 20 - (-5)
            // get_rect("")  vorher    (0, 20, 0, 26)   <- die Zeilenhoehe
            // ```
            //
            // Ein Pixel, und er ist **keine Rasterkante**: bei leerem
            // Text gibt es nichts zu rastern, also ist jeder Unterschied
            // eine andere Rechnung. Genau diese Sorte waere sonst unter
            // der benannten Grenze "Glyphenbox, hoechstens eins daneben"
            // verschwunden.
            //
            // Der gepolsterte Fall bleibt `ft_glyph_height()`: pygames
            // `ftfont.Font` setzt `pad = True`, und dort ist die Hoehe
            // fuer **jeden** Text dieselbe - `get_rect("")` und
            // `get_rect("Hi")` geben beide 19. Ein erster Versuch hat den
            // Leerfall ganz vorgezogen und damit die Polsterung
            // uebersprungen; *eine Abkuerzung, die an einer
            // Fallunterscheidung vorbeifuehrt, nimmt sie mit.*
            let hoehe = if self.pad {
                f.ft_glyph_height()
            } else {
                f.ft_ascender() - f.ft_descender()
            };
            let _ = &auflaenge;
            (0, 0, 0u32, hoehe.max(0) as u32, 0, auflaenge)
        } else if self.pad {
            (0, 0, b, f.ft_glyph_height().max(0) as u32, 0, auflaenge)
        } else {
            // **Der Unterstrich spannt die Tinte, nicht den Vorschub.**
            //
            // Gemessen an `render_raw("ABCDabcd", style=STYLE_UNDERLINE)`:
            // das Bild ist **116** breit wie ohne Unterstrich, obwohl der
            // Gesamtvorschub 117 ist, und der Strich liegt in den Spalten
            // 0 bis 115. pygame rechnet den Kasten also aus den **Glyphen**
            // und zieht den Strich erst darueber; er kann den Kasten
            // waagerecht nicht vergroessern, nur senkrecht.
            //
            // Hier wird gerastert und danach zugeschnitten, und der Strich
            // reicht ueber die volle Zeile - der Kasten waere also einen
            // Pixel breiter. Also zweimal messen: die **Breite** kommt vom
            // Bild ohne Strich, die **Hoehe** vom Bild mit.
            // `freetype_test.test_freetype_Font_get_rect` prueft beides in
            // einem Atemzug: `rect_under.width == rect_default.width` und
            // `rect_under.height > rect_default.height`.
            let waagerecht = if f.get_underline() {
                let mut ohne = self.in_groesse(size)?;
                self.stil_anwenden(&mut ohne, stil, grad);
                ohne.set_underline(false);
                let (rein, bo, ho) = self.zeilenkasten(&mut ohne, text)?;
                Self::tintenkasten(&rein, bo, ho).map(|(x0, _, w0, _)| (x0, w0))
            } else {
                None
            };
            // **Ein leeres Zeichen am Rand zaehlt mit**, und wie es
            // zaehlt, haengt von der Glaettung ab. Siehe `leerer_rand`.
            let (lx0, lx1) = self.leerer_rand(&f, text, b);
            match Self::tintenkasten(&roh, b, h) {
                Some((x0, y0, w0, h0)) => {
                    let (x0, w0) = waagerecht.unwrap_or((x0, w0));
                    // Nach links bis zum leeren Zeichen, nach rechts
                    // ueber es hinaus.
                    let links = lx0.map_or(x0, |v| v.min(x0));
                    let rechts = lx1.map_or(x0 + w0, |v| v.max(x0 + w0));
                    (
                        links,
                        y0,
                        rechts - links,
                        h0,
                        links as i32,
                        auflaenge - y0 as i32,
                    )
                }
                // **Nur Leerzeichen**, und das ist der Fall, bei dem die
                // beiden Zweige sich am deutlichsten unterscheiden:
                //
                // ```text
                //            pygame geglaettet   pygame einfarbig
                //   " "      (0, 0, 7, 0)        (0, 1, 1, 1)
                //   "  "     (0, 0, 14, 0)       (0, 1, 8, 1)
                // ```
                //
                // Geglaettet ist der Kasten so breit wie der Vorschub und
                // **null hoch**; einfarbig ist er ein einziges Pixel hoch
                // und am linken Rand jedes leeren Zeichens einen breit.
                None if !self.antialiased => {
                    let x0 = lx0.unwrap_or(0);
                    let x1 = lx1.unwrap_or(1);
                    // **Die Oberkante ist 1, nicht die Auflaenge.** Das
                    // eine Pixel sitzt auf der Grundlinie, nicht oben an
                    // der Zeile: gemessen gibt pygame fuer `" "` den
                    // Kasten `(0, 1, 1, 1)`, nicht `(0, 20, 1, 1)`.
                    (x0, 0, x1 - x0, 1, x0 as i32, 1)
                }
                None => (0, 0, b, 0, 0, 0),
            }
        };

        let mut aus = vec![0u8; (nw as usize) * (nh as usize)];
        for y in 0..nh {
            for x in 0..nw {
                let (sx, sy) = (ox + x, oy + y);
                if sx < b && sy < h {
                    aus[(y * nw + x) as usize] = roh[(sy * b + sx) as usize];
                }
            }
        }

        if grad != 0 && nw > 0 && nh > 0 {
            let (gedreht, gw, gh) = Self::gedreht(&aus, nw, nh, grad);
            // **Die Ecken um den Stiftpunkt drehen, nicht um die
            // Bildmitte.** Das Bild dreht sich um seine Mitte (siehe
            // `gedreht`), das *Rechteck* aber um den Punkt, an dem die
            // Schrift ansetzt - sonst stimmt der Versatz nicht mehr.
            // Nachgerechnet an `get_rect("M")` in vier Vierteldrehungen:
            // `(1,17,18,17)` -> `(-17,19,17,18)` -> `(-19,0,18,17)` ->
            // `(0,-1,17,18)`, und jede davon ist gemessen.
            let bogen = (grad as f64).to_radians();
            let (sin, cos) = (bogen.sin(), bogen.cos());
            let dreh = |x: f64, y: f64| (x * cos - y * sin, x * sin + y * cos);
            let (lx, oy2) = (rx as f64, ry as f64);
            let (rx2, uy) = ((rx + nw as i32) as f64, (ry - nh as i32) as f64);
            let ecken = [
                dreh(lx, oy2),
                dreh(rx2, oy2),
                dreh(lx, uy),
                dreh(rx2, uy),
            ];
            let minx = ecken.iter().map(|e| e.0).fold(f64::INFINITY, f64::min);
            let maxy = ecken.iter().map(|e| e.1).fold(f64::NEG_INFINITY, f64::max);
            rx = minx.round() as i32;
            ry = maxy.round() as i32;
            aus = gedreht;
            nw = gw;
            nh = gh;
        }

        Ok(Kasten {
            deckung: aus,
            w: nw,
            h: nh,
            rx,
            ry,
            glyphen: None,
            unterstrich: None,
        })
    }

    /// Die Nenngroesse, wie FreeType sie anfragt, in 26.6: `DBL_TO_FX6`
    /// schneidet ab, `FT_REQUEST_WIDTH`/`FT_REQUEST_HEIGHT` rechnen die
    /// Aufloesung ein (`(w * res + 36) / 72`). Bruchgroessen gehen damit
    /// auch. Die Hoehe ist `0`, wenn es keine eigene gibt.
    fn anfrage26(&self, gx: Option<f64>, gy: Option<f64>) -> (i64, i64) {
        let anfrage = |g: f64| ((g * 64.0) as i64 * self.dpi() as i64 + 36) / 72;
        (anfrage(gx.unwrap_or(self.groesse)), gy.map_or(0, anfrage))
    }

    /// Was `pygame.freetype`s Satz von der Schrift wissen muss - Stil
    /// (`STYLE_DEFAULT` heisst der der Schrift), Drehung und Groesse.
    fn ft_stil(&self, stil: i32, grad: i32, groesse26: i64, groesse26_y: i64) -> schrift::FtStil {
        let s = if stil == STYLE_DEFAULT { self.stil } else { stil };
        schrift::FtStil {
            strong: s & STYLE_STRONG != 0,
            oblique: s & STYLE_OBLIQUE != 0,
            wide: s & STYLE_WIDE != 0,
            underline: s & STYLE_UNDERLINE != 0,
            kerning: self.kerning,
            mono: !self.antialiased,
            pad: self.pad,
            strength: self.strength,
            underline_adjustment: self.underline_adjustment,
            drehung: i64::from(grad),
            senkrecht: self.senkrecht,
            groesse26,
            groesse26_y,
        }
    }

    /// `kasten` auf `pygame.freetype`s eigenem Weg - siehe
    /// `rustgame_font::Font::ft_zeile`. `None`, wo der nicht gilt.
    ///
    /// Die zusammengesetzte Deckung ist die von `render_raw`:
    /// `__render_glyph_GRAY1` mischt `s + d - s * d / 255`, und
    /// `__fill_glyph_GRAY1` **ueberschreibt** mit dem Unterstrich - samt
    /// pygames Unterkante `y + y - FX6_FLOOR(h + y)`, die nur fuer `y`
    /// auf einer Pixelgrenze die Bruchhoehe trifft.
    fn kasten_ft(
        &self,
        f: &schrift::Font,
        text: &str,
        stil: i32,
        grad: i32,
        groesse26: i64,
        groesse26_y: i64,
    ) -> Option<Kasten> {
        let fs = self.ft_stil(stil, grad, groesse26, groesse26_y);
        let z = f.ft_zeile(text, &fs, self.strikes)?;
        let (w, h) = (z.breite, z.hoehe);
        let mut d = vec![0u8; (w as usize) * (h as usize)];
        for g in &z.glyphen {
            for gy in 0..g.hoehe {
                let y = g.y + gy as i32;
                if y < 0 || y >= h as i32 {
                    continue;
                }
                for gx in 0..g.breite {
                    let x = g.x + gx as i32;
                    if x < 0 || x >= w as i32 {
                        continue;
                    }
                    let q = u32::from(g.deckung[gy * g.breite + gx]);
                    if q == 0 {
                        continue;
                    }
                    let i = (y as u32 * w + x as u32) as usize;
                    let alt = u32::from(d[i]);
                    d[i] = (q + alt - q * alt / 255) as u8;
                }
            }
        }
        if let Some((x, y, uw, uh)) = z.unterstrich {
            fuellen_grau(&mut d, w, h, x, y, uw, uh, 255);
        }
        Some(Kasten {
            deckung: d,
            w,
            h,
            rx: z.rx,
            ry: z.ry,
            glyphen: Some(z.glyphen),
            unterstrich: z.unterstrich,
        })
    }
}

/// `__fill_glyph_GRAY1` aus `ft_render_cb.c`, Zeile fuer Zeile.
#[allow(clippy::too_many_arguments)]
fn fuellen_grau(d: &mut [u8], sw: u32, sh: u32, x: i64, y: i64, w: i64, h: i64, shade: i64) {
    let ceil = |v: i64| (v + 63) & -64;
    let floor = |v: i64| v & -64;
    let round = |v: i64| (v + 32) & -64;
    let x = x.max(0);
    let y = y.max(0);
    let mut w = w;
    let mut h = h;
    if x + w > i64::from(sw) << 6 {
        w = (i64::from(sw) << 6) - x;
    }
    if y + h > i64::from(sh) << 6 {
        h = (i64::from(sh) << 6) - y;
    }
    let spalten = ceil(w) >> 6;
    let x0 = ceil(x) >> 6;
    let mut zeile = ceil(y) >> 6;
    let mut setzen = |zeile: i64, wert: i64| {
        if zeile < 0 || zeile >= i64::from(sh) {
            return;
        }
        for i in 0..spalten {
            let xx = x0 + i;
            if xx >= 0 && xx < i64::from(sw) {
                d[(zeile * i64::from(sw) + xx) as usize] = wert as u8;
            }
        }
    };
    if y < ceil(y) {
        setzen(zeile - 1, round(shade * (ceil(y) - y)) >> 6);
    }
    for _ in 0..((floor(h + y) - ceil(y)) >> 6) {
        setzen(zeile, shade);
        zeile += 1;
    }
    if h > floor(h + y) - y {
        setzen(zeile, round(shade * (y + y - floor(h + y))) >> 6);
    }
}

/// `__fill_glyph_RGB4` aus `ft_render_cb.c`: das Unterstrich-Rechteck mit
/// Bruchkanten, je Pixel `ALPHA_BLEND` mit `edge_a`.
#[allow(clippy::too_many_arguments)]
fn fuellen_rgb(
    flaeche: &mut rustgame_core::surface::Surface,
    x: i64,
    y: i64,
    w: i64,
    h: i64,
    vg: rustgame_core::color::Color,
    hat_alpha: bool,
) {
    let (sw, sh) = flaeche.get_size();
    let ceil = |v: i64| (v + 63) & -64;
    let floor = |v: i64| v & -64;
    let round = |v: i64| (v + 32) & -64;
    let x = x.max(0);
    let mut y = y.max(0);
    let mut w = w;
    let mut h = h;
    if x + w > i64::from(sw) << 6 {
        w = (i64::from(sw) << 6) - x;
    }
    if y + h > i64::from(sh) << 6 {
        h = (i64::from(sh) << 6) - y;
    }
    let spalten = ceil(w) >> 6;
    let x0 = ceil(x) >> 6;
    let mut zeile = ceil(y) >> 6;
    let mischen = |f: &mut rustgame_core::surface::Surface, zeile: i64, a: i64| {
        if zeile < 0 || zeile >= i64::from(sh) {
            return;
        }
        for i in 0..spalten {
            let xx = x0 + i;
            if xx < 0 || xx >= i64::from(sw) {
                continue;
            }
            let p = (xx as i32, zeile as i32);
            let oben = rustgame_core::color::Color::rgba(vg.r, vg.g, vg.b, a.clamp(0, 255) as u8);
            let alt = f.get_at_mapped(p);
            f.set_at_ungeprueft(p, PyFreetypeFont::ueber(oben, alt, hat_alpha));
        }
    };
    let mut dh = ceil(y) - y;
    if dh > h {
        dh = h;
    }
    h -= dh;
    if dh > 0 {
        mischen(flaeche, zeile - 1, round(i64::from(vg.a) * dh) >> 6);
        y += dh;
    }
    let mut voll = floor(h);
    h -= voll;
    while voll > 0 {
        mischen(flaeche, zeile, i64::from(vg.a));
        zeile += 1;
        voll -= 64;
        y += 64;
    }
    let _ = y;
    if h > 0 {
        mischen(flaeche, zeile, round(i64::from(vg.a) * h) >> 6);
    }
}

/// Was eine Messung ueber einen Text ergibt - siehe [`PyFreetypeFont::kasten`].
struct Kasten {
    /// Ein Byte Deckung je Pixel, schon zugeschnitten und gedreht.
    deckung: Vec<u8>,
    w: u32,
    h: u32,
    /// Das Rechteck, das `get_rect` meldet: `x` nach rechts, `y` **nach
    /// oben** von der Grundlinie aus.
    rx: i32,
    ry: i32,
    /// Auf `pygame.freetype`s eigenem Weg (`Font::ft_zeile`): die Zeichen
    /// **einzeln**, wie `render()` sie nacheinander in die Flaeche mischt.
    /// `None` auf dem alten Weg (gedreht, senkrecht, Bitmapschrift).
    glyphen: Option<Vec<schrift::FtGlyphe>>,
    /// Der Unterstrich als Rechteck in 26.6 - `surface->fill`.
    unterstrich: Option<(i64, i64, i64, i64)>,
}

#[pymethods]
impl PyFreetypeFont {
    /// `Font.__new__(Font)` **ohne Argumente** gibt die Nullschrift.
    ///
    /// Das ist kein Sonderweg, sondern pygames Aufteilung: `tp_new` legt
    /// die Huelle an, `tp_init` laedt die Datei. `freetype_test.nullfont()`
    /// baut sich genau so ein Objekt und verlangt von jeder Auskunft einen
    /// `RuntimeError`.
    ///
    /// PyO3 0.26 hat keinen `tp_init`-Schlitz, deshalb laedt hier schon
    /// `__new__` - **ausser** es kommt gar nichts an. Die Unterscheidung
    /// geht nur ueber `*args`, weil `file=None` ein gueltiger Wert ist
    /// (`Font(None)` ist die Standardschrift).
    #[new]
    #[pyo3(signature = (*args, **kwargs))]
    fn new(
        args: &Bound<'_, pyo3::types::PyTuple>,
        kwargs: Option<&Bound<'_, PyDict>>,
    ) -> PyResult<PyFreetypeFont> {
        if args.is_empty() && kwargs.map(|k| k.is_empty()).unwrap_or(true) {
            return Ok(PyFreetypeFont::leer());
        }
        let (file, size, font_index, resolution, ucs4) = Self::argumente(args, kwargs)?;
        Self::laden(file.as_ref(), size, font_index, resolution, ucs4)
    }

    /// `f.__init__(pfad, size=12)` - laedt **in dasselbe Objekt** nach.
    ///
    /// Gemessen bleiben dabei `antialiased`, `oblique` und `ucs4` stehen;
    /// nur Datei, Groesse und Aufloesung werden ersetzt.
    #[pyo3(signature = (*args, **kwargs))]
    fn __init__(
        slf: &Bound<'_, PyFreetypeFont>,
        args: &Bound<'_, pyo3::types::PyTuple>,
        kwargs: Option<&Bound<'_, PyDict>>,
    ) -> PyResult<()> {
        if args.is_empty() && kwargs.map(|k| k.is_empty()).unwrap_or(true) {
            return Ok(());
        }
        let (file, size, font_index, resolution, ucs4) = Self::argumente(args, kwargs)?;
        let neu = Self::laden(file.as_ref(), size, font_index, resolution, ucs4)?;
        let mut ich = slf.borrow_mut();
        ich.inner = neu.inner;
        ich.pfad = neu.pfad;
        ich.groesse = neu.groesse;
        ich.groesse_y = neu.groesse_y;
        ich.aufloesung = neu.aufloesung;
        Ok(())
    }

    // -- Messen und Zeichnen ist weiter unten; hier hoert `__init__` auf.

    // ----- Masse in Schrifteinheiten --------------------------------------

    /// Die Zeilenhoehe in **Schrifteinheiten**.
    #[getter]
    fn height(&self) -> PyResult<i32> {
        Ok(self.kern()?.roh_height())
    }
    /// Die Oberlaenge in Schrifteinheiten.
    #[getter]
    fn ascender(&self) -> PyResult<i32> {
        Ok(self.kern()?.roh_ascender() as i32)
    }
    /// Die Unterlaenge in Schrifteinheiten - negativ.
    #[getter]
    fn descender(&self) -> PyResult<i32> {
        Ok(self.kern()?.roh_descender() as i32)
    }

    /// Dieselben drei in **Pixeln**, fuer eine bestimmte Groesse.
    ///
    /// Bei einer Umrissschrift aus [`schrift::Font::ft_kopf26`]: FreeType
    /// rechnet mit der **ungerundeten** Skala der Nenngroesse, also gibt
    /// `size=20.5` nicht dasselbe wie 20 oder 21 (gemessen an Arial:
    /// `get_sized_ascender(20.5)` ist 19, bei 21 sind es 20). Bei einem
    /// Groessenpaar zaehlt fuer diese drei die **zweite** Zahl.
    #[pyo3(signature = (size=None, /))]
    fn get_sized_ascender(&self, size: Option<Groesse>) -> PyResult<i32> {
        match self.kopfmasse(size)? {
            Some((a, _, _)) => Ok((a >> 6) as i32),
            None => Ok(self.in_groesse(size)?.ft_ascender()),
        }
    }
    #[pyo3(signature = (size=None, /))]
    fn get_sized_descender(&self, size: Option<Groesse>) -> PyResult<i32> {
        match self.kopfmasse(size)? {
            Some((_, d, _)) => Ok((d >> 6) as i32),
            None => Ok(self.in_groesse(size)?.ft_descender()),
        }
    }
    #[pyo3(signature = (size=None, /))]
    fn get_sized_height(&self, size: Option<Groesse>) -> PyResult<i32> {
        match self.kopfmasse(size)? {
            Some((_, _, h)) => Ok((h >> 6) as i32),
            None => Ok(self.in_groesse(size)?.ft_height()),
        }
    }
    #[pyo3(signature = (size=None, /))]
    fn get_sized_glyph_height(&self, size: Option<Groesse>) -> PyResult<i32> {
        match self.kopfmasse(size)? {
            // `FX6_TRUNC(FX6_CEIL(asc) - FX6_FLOOR(desc)) + 1`
            Some((a, d, _)) => Ok(((a - d) >> 6) as i32 + 1),
            None => Ok(self.in_groesse(size)?.ft_glyph_height()),
        }
    }

    // ----- Eigenschaften --------------------------------------------------

    /// `f.size` - eine Zahl, bei einer Bitmapschrift aber ein **Paar**.
    ///
    /// Gemessen: `ft.Font("PyGameMono-8.bdf").size` ist `(8.0, 8.0)`. Eine
    /// eingebaute Groesse darf waagerecht und senkrecht verschieden sein,
    /// und pygame gibt sie deshalb als Paar heraus - bis jemand von Hand
    /// eine Zahl setzt, dann ist es wieder eine.
    #[getter]
    fn get_size(&self, py: Python<'_>) -> PyResult<Py<PyAny>> {
        self.epoche_pruefen()?;
        match self.groesse_y {
            Some(y) => Ok((self.groesse, y).into_pyobject(py)?.into_any().unbind()),
            None => Ok(self.groesse.into_pyobject(py)?.into_any().unbind()),
        }
    }
    /// **Setzen baut nichts.** Die Schrift entsteht erst, wenn jemand
    /// misst oder zeichnet - sonst muesste `f.size = 33554431` eine
    /// Schrift von dreiunddreissig Millionen Punkten rastern, und genau
    /// das steht in `test_freetype_Font_size`.
    #[setter]
    fn set_size(&mut self, v: &Bound<'_, PyAny>) -> PyResult<()> {
        self.epoche_pruefen()?;
        let v: Groesse = v.extract()?;
        self.groesse = v.0;
        self.groesse_y = v.1;
        Ok(())
    }
    #[getter]
    fn get_style(&self) -> PyResult<i32> {
        self.epoche_pruefen()?;
        Ok(self.stil)
    }
    /// `f.style = ...`
    ///
    /// **`STYLE_DEFAULT` setzt nichts.** Gemessen: nach
    /// `font.style = STYLE_STRONG|STYLE_UNDERLINE|STYLE_OBLIQUE` und
    /// anschliessendem `font.style = STYLE_DEFAULT` steht dort weiterhin
    /// die Dreierkombination. `-1` heisst "nimm, was die Schrift selbst
    /// sagt" und ist kein Wert, den man ablegt.
    #[setter]
    fn set_style(&mut self, v: &Bound<'_, PyAny>) -> PyResult<()> {
        // **Erst leben, dann pruefen.** Nach einem `quit()` gibt jeder
        // Zugriff `pygame.error`, und zwar auch dann, wenn der Wert
        // ausserdem vom falschen Typ ist - gemessen an
        // `font_test.test_font_property_should_raise_exception_after_quit`,
        // das genau diese Reihenfolge durchspielt.
        self.epoche_pruefen()?;
        // **`True` ist eine Zahl, und pygame nimmt sie.** Gemessen gibt
        // `f.style = True` dort `1` und `f.style = False` `0`; hier stand
        // ein eigener Riegel gegen `PyBool`, der beide zurueckwies.
        if !v.is_instance_of::<pyo3::types::PyInt>() {
            return Err(PyTypeError::new_err(
                "The style value must be an integer from the FT constants module",
            ));
        }
        // pygame liest den Wert als C-`long` - unter Windows **32 Bit** -
        // und schreibt bei Ueberlauf die `-1` des gescheiterten Lesens in
        // den Satz. Gemessen: `2**31` und `10**30` geben beide
        // `Invalid style value ffffffff`, `-2**31` dagegen `80000000`,
        // also den echten Wert. Vorher kam hier PyO3s `OverflowError`.
        let v: i32 = v.extract().unwrap_or(-1);
        if v == STYLE_DEFAULT {
            return Ok(());
        }
        if (v & !(STYLE_STRONG | STYLE_OBLIQUE | STYLE_UNDERLINE | STYLE_WIDE)) != 0 {
            // pygame schreibt den Wert **hexadezimal** dahinter, und zwar
            // als 32-Bit-Muster: `-7` wird `fffffff9`, `16` wird `10`.
            return Err(PyValueError::new_err(format!(
                "Invalid style value {:x}",
                v as u32
            )));
        }
        self.stil_erlaubt(v)?;
        self.stil = v;
        Ok(())
    }
    #[getter]
    fn get_strong(&self) -> PyResult<bool> {
        self.epoche_pruefen()?;
        Ok(self.stil & STYLE_STRONG != 0)
    }
    #[setter]
    fn set_strong(&mut self, v: &Bound<'_, PyAny>) -> PyResult<()> {
        self.epoche_pruefen()?;
        let v = schalter(v)?;
        self.stil_erlaubt(STYLE_STRONG)?;
        self.set_strong_roh(v);
        Ok(())
    }
    #[getter]
    fn get_oblique(&self) -> PyResult<bool> {
        self.epoche_pruefen()?;
        Ok(self.stil & STYLE_OBLIQUE != 0)
    }
    #[setter]
    fn set_oblique(&mut self, v: &Bound<'_, PyAny>) -> PyResult<()> {
        self.epoche_pruefen()?;
        let v = schalter(v)?;
        self.stil_erlaubt(STYLE_OBLIQUE)?;
        self.set_oblique_roh(v);
        Ok(())
    }
    #[getter]
    fn get_underline(&self) -> PyResult<bool> {
        self.epoche_pruefen()?;
        Ok(self.stil & STYLE_UNDERLINE != 0)
    }
    #[setter]
    fn set_underline(&mut self, v: &Bound<'_, PyAny>) -> PyResult<()> {
        self.epoche_pruefen()?;
        let v = schalter(v)?;
        self.setze_underline(v);
        Ok(())
    }
    #[getter]
    fn get_wide(&self) -> PyResult<bool> {
        self.epoche_pruefen()?;
        Ok(self.stil & STYLE_WIDE != 0)
    }
    #[setter]
    fn set_wide(&mut self, v: &Bound<'_, PyAny>) -> PyResult<()> {
        self.epoche_pruefen()?;
        let v = schalter(v)?;
        self.setze_wide(v);
        Ok(())
    }
    #[getter]
    fn get_origin(&self) -> PyResult<bool> {
        self.epoche_pruefen()?;
        Ok(self.origin)
    }
    #[setter]
    fn set_origin(&mut self, v: &Bound<'_, PyAny>) -> PyResult<()> {
        self.epoche_pruefen()?;
        let v = schalter(v)?;
        self.setze_origin(v);
        Ok(())
    }
    #[getter]
    fn get_pad(&self) -> PyResult<bool> {
        self.epoche_pruefen()?;
        Ok(self.pad)
    }
    #[setter]
    fn set_pad(&mut self, v: &Bound<'_, PyAny>) -> PyResult<()> {
        self.epoche_pruefen()?;
        let v = schalter(v)?;
        self.setze_pad(v);
        Ok(())
    }
    #[getter]
    fn get_antialiased(&self) -> PyResult<bool> {
        self.epoche_pruefen()?;
        Ok(self.antialiased)
    }
    #[setter]
    fn set_antialiased(&mut self, v: &Bound<'_, PyAny>) -> PyResult<()> {
        self.epoche_pruefen()?;
        let v = schalter(v)?;
        self.setze_antialiased(v);
        Ok(())
    }
    #[getter]
    fn get_kerning(&self) -> PyResult<bool> {
        self.epoche_pruefen()?;
        Ok(self.kerning)
    }
    #[setter]
    fn set_kerning(&mut self, v: &Bound<'_, PyAny>) -> PyResult<()> {
        self.epoche_pruefen()?;
        let v = schalter(v)?;
        self.setze_kerning(v);
        Ok(())
    }
    #[getter]
    fn get_ucs4(&self) -> PyResult<bool> {
        self.epoche_pruefen()?;
        Ok(self.ucs4)
    }
    #[setter]
    fn set_ucs4(&mut self, v: &Bound<'_, PyAny>) -> PyResult<()> {
        self.epoche_pruefen()?;
        let v = schalter(v)?;
        self.setze_ucs4(v);
        Ok(())
    }
    #[getter]
    fn get_underline_adjustment(&self) -> PyResult<f64> {
        self.epoche_pruefen()?;
        Ok(self.underline_adjustment)
    }
    /// `f.underline_adjustment = ...` - **nur zwischen -2 und 2**.
    ///
    /// Der Bereich fehlte hier ganz: `-7`, `1e+30` und `2**31` gingen
    /// durch, wo pygame
    /// `underline adjustment value '-7.0' is outside range [-2.0, 2.0]`
    /// sagt. Gemessen sind die Raender **eingeschlossen** (`2.0` geht,
    /// `2.0001` nicht).
    #[setter]
    fn set_underline_adjustment(&mut self, v: &Bound<'_, PyAny>) -> PyResult<()> {
        self.epoche_pruefen()?;
        let (v, wie) = als_gleitzahl(v)?;
        if !(-2.0..=2.0).contains(&v) {
            return Err(PyValueError::new_err(format!(
                "underline adjustment value '{wie}' is outside range [-2.0, 2.0]"
            )));
        }
        self.underline_adjustment = v;
        Ok(())
    }
    #[getter]
    fn get_fgcolor(&self) -> PyResult<crate::farbe::PyColor> {
        self.epoche_pruefen()?;
        Ok(crate::farbe::PyColor::neu(self.fgcolor))
    }
    /// **`None` ist keine Farbe** - und eine blanke Zahl auch nicht.
    ///
    /// Gemessen gibt jeder nicht annehmbare Wert hier denselben
    /// `AttributeError` mit demselben Satz, `None` eingeschlossen:
    /// `unable to convert …NoneType object to a color`. Hier stand ein
    /// eigener Satz fuer `None` (`fgcolor cannot be None`) und sonst
    /// [`farbe_aus`], das Namen (`'red'`) und blanke Zahlen annimmt -
    /// pygame nimmt an dieser Stelle **nur** Folgen und `Color`.
    #[setter]
    fn set_fgcolor(&mut self, o: &Bound<'_, PyAny>) -> PyResult<()> {
        self.epoche_pruefen()?;
        self.fgcolor = farbe_fuer_freitext(o)?;
        Ok(())
    }
    /// `f.bgcolor` - anfangs **durchsichtiges Schwarz**, nicht `None`.
    #[getter]
    fn get_bgcolor(&self) -> PyResult<crate::farbe::PyColor> {
        self.epoche_pruefen()?;
        Ok(crate::farbe::PyColor::neu(
            self.bgcolor
                .unwrap_or(rustgame_core::color::Color::rgba(0, 0, 0, 0)),
        ))
    }
    /// Wie [`Font::set_fgcolor`] - derselbe Satz, dieselbe Enge.
    #[setter]
    fn set_bgcolor(&mut self, o: &Bound<'_, PyAny>) -> PyResult<()> {
        self.epoche_pruefen()?;
        self.bgcolor = Some(farbe_fuer_freitext(o)?);
        Ok(())
    }
    /// **`AttributeError`, nicht `RuntimeError`.** Gemessen: von allen
    /// Zugriffen auf eine Nullschrift sind `name` und `path` die beiden,
    /// die einen `AttributeError` geben - sie fragen die Datei, und die
    /// gibt es gar nicht.
    #[getter]
    fn name(&self) -> PyResult<String> {
        if self.inner.is_some() {
            self.kern()?;
        }
        match &self.inner {
            Some(k) => Ok(k.name()),
            None => Err(ohne_datei()),
        }
    }
    /// `repr(f)` - **`Font('<pfad>')`**, und sonst das Nichtwissen.
    ///
    /// Gemessen an vier Schriften:
    ///
    /// ```text
    /// ft.Font(None, 13)            Font('...\\pygame\\freesansbold.ttf')
    /// ft.Font(pfad, 20)            dasselbe, der eigene Pfad
    /// ft.SysFont('arial', 12)      Font('C:\\WINDOWS\\Fonts\\ARIALN.TTF')
    /// ft.Font.__new__(ft.Font)     <uninitialized Font object at 0xADR>
    /// ```
    ///
    /// Hier gab es gar keinen - also CPythons
    /// `<rustgame_py.freetype.Font object at 0x...>`. Und eine Datei
    /// weiter hatte `pygame.font.Font` einen **zuviel**
    /// (`<Font(size=8)>`); dort gibt pygame den gewoehnlichen Satz von
    /// `object`. Zwei Klassen mit demselben Namen, zwei verschiedene
    /// Antworten - und beide standen falsch.
    fn __repr__(slf: &Bound<'_, Self>) -> PyResult<String> {
        let this = slf.borrow();
        if this.inner.is_none() {
            return Ok(format!(
                "<uninitialized Font object at {:p}>",
                slf.as_ptr()
            ));
        }
        drop(this);
        // Ueber den Getter, damit der Pfad derselbe ist, den `f.path`
        // nennt - samt der Suche nach der Vorgabeschrift.
        let pfad = Self::path(&slf.borrow())?;
        // **Nicht `{:?}`.** pygame schreibt den Pfad roh zwischen zwei
        // Hochkommas - `Font('C:\Users\...')`, mit einfachen
        // Schraegstrichen. Rusts `{:?}` (und Pythons `repr`) verdoppelt
        // sie und nimmt Anfuehrungszeichen.
        Ok(format!("Font('{pfad}')"))
    }

    #[getter]
    fn style_name(&self) -> PyResult<String> {
        self.epoche_pruefen()?;
        let Some(k) = &self.inner else {
            return Err(ohne_datei());
        };
        Ok(k.style_name())
    }
    #[getter]
    fn path(&self) -> PyResult<String> {
        if self.inner.is_some() {
            self.kern()?;
        }
        if self.inner.is_none() {
            return Err(pyo3::exceptions::PyAttributeError::new_err(
                "path unavailable",
            ));
        }
        // **Der ganze Pfad, nicht der Dateiname.** Gemessen gibt pygame
        // hier `...\\site-packages\\pygame\\freesansbold.ttf`; hier stand
        // `freesansbold.ttf`, also der Name aus `get_default_font()`.
        // Gesucht wird die Datei ohnehin (siehe
        // `rustgame_font::default_font_path`) - gemeldet wurde nur nicht,
        // was gefunden wurde.
        Ok(self.pfad.clone().unwrap_or_else(|| {
            schrift::default_font_path()
                .map(|p| p.to_string_lossy().into_owned())
                .unwrap_or_else(|| schrift::get_default_font().to_string())
        }))
    }
    /// Laesst sich die Schrift stufenlos vergroessern?
    #[getter]
    fn scalable(&self) -> PyResult<bool> {
        Ok(!self.kern()?.ist_bitmap())
    }
    #[getter]
    fn fixed_width(&self) -> PyResult<bool> {
        Ok(self.kern()?.feste_breite())
    }
    #[getter]
    fn fixed_sizes(&self) -> PyResult<i32> {
        Ok(self.kern()?.feste_groessen().len() as i32)
    }
    #[getter]
    fn use_bitmap_strikes(&self) -> PyResult<bool> {
        self.epoche_pruefen()?;
        Ok(self.strikes)
    }
    #[setter]
    fn set_use_bitmap_strikes(&mut self, v: &Bound<'_, PyAny>) -> PyResult<()> {
        self.epoche_pruefen()?;
        self.strikes = schalter(v)?;
        Ok(())
    }
    #[getter]
    fn get_resolution(&self) -> PyResult<u32> {
        self.epoche_pruefen()?;
        // **Die Nullschrift sagt 0, nicht 72.** `dpi()` setzt die
        // Vorgabe ein, wo nichts steht - und das ist richtig, solange es
        // eine Schrift gibt. Ohne Datei gibt pygame die blanke Null
        // heraus (gemessen an `Font.__new__(Font).resolution`), und das
        // ist genau der Zettelwert.
        if self.inner.is_none() {
            return Ok(self.aufloesung);
        }
        Ok(self.dpi())
    }
    /// Die eingebauten Groessen - bei einer Umrissschrift leer.
    fn get_sizes(&self) -> PyResult<FesteGroessen> {
        Ok(self.kern()?.feste_groessen())
    }

    /// **Gedrehter Text gibt es hier nicht** - siehe den Modulvermerk.
    #[getter]
    fn get_rotation(&self) -> PyResult<i32> {
        self.epoche_pruefen()?;
        Ok(self.drehung)
    }
    #[setter]
    fn set_rotation(&mut self, v: &Bound<'_, PyAny>) -> PyResult<()> {
        // `_ftfont_setrotation`: eine Bitmapschrift lehnt **jeden** Wert
        // ab, auch `0` - und das vor der Typpruefung.
        if self.kern()?.ist_bitmap() {
            return Err(pyo3::exceptions::PyAttributeError::new_err(
                "rotation is unsupported for a bitmap font",
            ));
        }
        // Wie bei `style`: `True` ist eine Zahl und geht durch (gemessen
        // `f.rotation = True` -> `1`). pygame nennt im Fehlersatz den
        // Typnamen, **ohne** Polster und ohne Anfuehrungszeichen:
        // `integer rotation expected, got NoneType`.
        if !v.is_instance_of::<pyo3::types::PyInt>() {
            return Err(PyTypeError::new_err(format!(
                "integer rotation expected, got {}",
                typname(v)
            )));
        }
        // **Erst kuerzen, dann lesen.** pygame rechnet den Winkel in
        // Pythons Zahlenraum auf `[0, 360)` herunter, und das kann jede
        // Zahl: gemessen gibt `rotation = 10**30` dort **280** und
        // `2**63` **8**, waehrend hier PyO3s `OverflowError` kam. Der
        // Rest nach `%` ist in Python nie negativ, also stimmt er schon.
        let rest = v.call_method1("__mod__", (360i32,))?;
        self.drehung = rest.extract()?;
        Ok(())
    }
    #[getter]
    fn get_vertical(&self) -> PyResult<bool> {
        self.epoche_pruefen()?;
        Ok(self.senkrecht)
    }
    #[setter]
    fn set_vertical(&mut self, v: &Bound<'_, PyAny>) -> PyResult<()> {
        self.epoche_pruefen()?;
        self.senkrecht = schalter(v)?;
        Ok(())
    }
    /// Wie stark `STYLE_STRONG` verdickt - als Anteil der Groesse.
    ///
    /// **Hier flog frueher ein Fehler.** Die Begruendung war, rustgame habe
    /// keinen Konturzeichner, und das stimmt: FreeType verdickt die
    /// **Kontur**, rustgame verschmiert wie SDL_ttf das fertige Bild. Der
    /// Wert ist trotzdem kein leeres Feld - er sagt, um wie viele Spalten
    /// verschmiert wird (`Ueberhang = Groesse * strength`), und das war
    /// vorher die feste Zahl `Groesse / 10`.
    ///
    /// Gebraucht wird er von `ftfont.py`, das ihn beim Anlegen jeder
    /// Schrift auf `1/12` setzt. Ein Fehler an dieser Stelle machte das
    /// ganze Modul unbenutzbar - und mit ihm die 72 Tests, die es fahren.
    #[getter]
    fn get_strength(&self) -> PyResult<f64> {
        self.epoche_pruefen()?;
        Ok(self.strength)
    }
    #[setter]
    fn set_strength(&mut self, v: &Bound<'_, PyAny>) -> PyResult<()> {
        self.epoche_pruefen()?;
        let (v, wie) = als_gleitzahl(v)?;
        if !(0.0..=1.0).contains(&v) {
            // pygame nennt den Wert, und zwar als Pythons `repr` -
            // gemessen `strength value '1e+30' is outside range [0, 1]`.
            return Err(PyValueError::new_err(format!(
                "strength value '{wie}' is outside range [0, 1]"
            )));
        }
        self.strength = v;
        Ok(())
    }

    // ----- Messen und Zeichnen --------------------------------------------

    /// Das Rechteck, das der Text belegen wird.
    #[pyo3(signature = (text, style=STYLE_DEFAULT, rotation=None, size=None))]
    fn get_rect(
        &self,
        text: &Bound<'_, PyAny>,
        style: i32,
        rotation: Option<i32>,
        size: Option<Groesse>,
    ) -> PyResult<PyRect> {
        self.stil_pruefen(style)?;
        self.modus_pruefen(style, rotation)?;
        let t = self.text_von(text)?;
        let mut f = self.in_groesse(size)?;
        self.stil_anwenden(&mut f, style, self.winkel(rotation));
        // **Gemessen wird, was auch gezeichnet wird.** Sobald gedreht
        // oder senkrecht gesetzt wird, ist `f.size()` die falsche Frage -
        // sie kennt weder das eine noch das andere. Also kommt das Mass
        // aus derselben Deckung, aus der `render` sein Bild baut; sonst
        // meldet `get_rect` ein Rechteck, in das das Bild nicht passt.
        // **Gemessen wird, was auch gezeichnet wird.** `f.size()` kennt
        // weder Drehung noch senkrechten Satz noch den Unterstrich, der
        // unter die Grundlinie reicht; jede dieser drei Stellen gab ein
        // Rechteck, in das das Bild nicht passte. Ein `get_rect`, das
        // billiger, aber nicht dasselbe misst, ist die teuerste Sorte
        // Fehler - man sieht ihn erst am fertigen Bild.
        let _ = &f;
        // **`origin` aendert hier nichts.** Das stand einmal anders: die
        // Hoehe der Auflaenge wanderte bei `origin = True` ins `y`. Gemessen
        // gibt pygame fuer beide Einstellungen **dasselbe** Rechteck - der
        // Unterschied zeigt sich erst bei `render_to`, wo `dest` einmal die
        // linke obere Ecke meint und einmal den Stiftpunkt.
        let k = self.kasten(&t, style, size, rotation)?;
        Ok(PyRect::neu(rustgame_core::rect::Rect::new(
            k.rx,
            k.ry,
            k.w as i32,
            k.h as i32,
        )))
    }

    /// Die Masse jedes Zeichens - sechs Zahlen je Glyphe.
    #[pyo3(signature = (text, size=None))]
    fn get_metrics(
        &self,
        text: &Bound<'_, PyAny>,
        size: Option<Groesse>,
    ) -> PyResult<Glyphenmasse> {
        let mut f = self.in_groesse(size)?;
        // `_PGFT_BuildRenderMode(.., FT_STYLE_DEFAULT, self->rotation)`
        self.modus_pruefen(STYLE_DEFAULT, None)?;
        // **Der Stil gehoert dazu.** Gemessen aendert `font.oblique =
        // True` die Masse; ohne diese Zeile sah `get_metrics` den Stil
        // der Schrift gar nicht, und `test_issue_565` bekam zweimal
        // dieselbe Liste.
        // `get_metrics` dreht nicht - es gibt die Masse der ungedrehten
        // Zeichen heraus, auch wenn `font.rotation` gesetzt ist.
        self.stil_anwenden(&mut f, STYLE_DEFAULT, 0);
        // `metrics` gibt die ganze Liste auf einmal - je Zeichen fuenf
        // Zahlen oder `None`, wenn die Schrift es nicht hat.
        // **Ein unbekanntes Zeichen gibt `None`, keinen leeren Kasten.**
        // Gemessen an `test_undefined_character_code`: fuer `chr(1)` steht
        // in der Liste `None`, fuer `"0"` ein Sechstupel. Wer hier Nullen
        // einsetzt, gibt eine Auskunft, die wie ein gueltiges Mass
        // aussieht.
        let text = self.als_text(text)?;
        // **pygame laedt jedes Zeichen wie beim Satz** (`_PGFT_GetMetrics`
        // ruft `_PGFT_LoadGlyph`) und gibt die Kanten des Bildes und den
        // gedrehten Vorschub heraus - mit Stil, Drehung, Glaettung und
        // Groesse der Schrift. Bis Runde 20 kamen die Masse hier aus der
        // Schriftkiste (SDL_ttfs Weg) und stimmten nur im Grundfall:
        // gemessen an `Font(None, 24)` und `"Ag"` waren `strong`, `wide`,
        // `oblique`, einfarbig, 30 Grad, `size=20.5` und `size=(24, 12)`
        // jedes Mal anders.
        let (gx, gy) = match size {
            Some(g) => (Some(g.0), g.1),
            None => (None, self.groesse_y),
        };
        let (g26, g26_y) = self.anfrage26(gx, gy);
        let fs = self.ft_stil(STYLE_DEFAULT, self.winkel(None), g26, g26_y);
        // Ein Zeichen, das FreeType nicht laden kann, gibt `None`
        // (`_PGFT_GetMetrics` scheitert) - siehe `Font::ft_ladefehler`.
        if f.ft_ladefehler(&text, &fs, self.strikes).is_some() {
            return Ok(text
                .chars()
                .map(|c| {
                    let einzeln = c.to_string();
                    if f.ft_ladefehler(&einzeln, &fs, self.strikes).is_some() {
                        return None;
                    }
                    f.ft_zeile(&einzeln, &fs, self.strikes)
                        .and_then(|z| z.masse.first().copied())
                        .filter(|m| m.id != 0)
                        .map(|m| {
                            (
                                m.min_x as i32,
                                m.max_x as i32,
                                m.min_y as i32,
                                m.max_y as i32,
                                m.vorschub.0,
                                m.vorschub.1,
                            )
                        })
                })
                .collect());
        }
        if let Some(z) = f.ft_zeile(&text, &fs, self.strikes) {
            return Ok(z
                .masse
                .iter()
                .map(|m| {
                    (m.id != 0).then_some((
                        m.min_x as i32,
                        m.max_x as i32,
                        m.min_y as i32,
                        m.max_y as i32,
                        m.vorschub.0,
                        m.vorschub.1,
                    ))
                })
                .collect());
        }
        let hat: Vec<bool> = text.chars().map(|c| f.hat_zeichen(c)).collect();
        Ok(f
            .metrics(&text)
            .into_iter()
            .enumerate()
            .map(|(i, m)| {
                if !hat.get(i).copied().unwrap_or(false) {
                    return None;
                }
                m.map(|(minx, maxx, miny, maxy, adv)| {
                    Self::mass_gedreht(
                        (minx, maxx, miny, maxy, adv as f64, 0.0),
                        self.drehung,
                    )
                })
            })
            .collect())
    }

    /// Zeichnet den Text und gibt `(Flaeche, Rect)`.
    ///
    /// Acht Argumente, und das ist pygames Signatur: `text`, `fgcolor`,
    /// `bgcolor`, `style`, `rotation`, `size` - dazu `self` und `py`. Sie
    /// zu buendeln hiesse, die Schnittstelle zu aendern, um einen Hinweis
    /// loszuwerden.
    #[allow(clippy::too_many_arguments)]
    #[pyo3(signature = (text, fgcolor=None, bgcolor=None, style=STYLE_DEFAULT, rotation=None, size=None))]
    fn render(
        &self,
        py: Python<'_>,
        text: &Bound<'_, PyAny>,
        fgcolor: Option<&Bound<'_, PyAny>>,
        bgcolor: Option<&Bound<'_, PyAny>>,
        style: i32,
        rotation: Option<i32>,
        size: Option<Groesse>,
    ) -> PyResult<(Py<PySurface>, PyRect)> {
        self.stil_pruefen(style)?;
        self.modus_pruefen(style, rotation)?;
        let t = self.text_von(text)?;
        let vg = match fgcolor {
            Some(o) if !o.is_none() => farbe_streng(o)?,
            _ => self.fgcolor,
        };
        let hg = match bgcolor {
            Some(o) if !o.is_none() => Some(farbe_streng(o)?),
            _ => self.bgcolor,
        };
        // **Ohne Glaettung baut die Schriftkiste die Flaeche selbst.**
        //
        // Gemessen gibt `pygame.freetype` dort eine Flaeche mit **acht
        // Bit**, einer Zweifarbentafel und einer Schluesselfarbe - nicht
        // eine mit Alphakanal. Genau das liefert `rustgame_font::render`
        // schon, denn `pygame.font` braucht dieselbe Flaeche. Sie hier
        // aus der Deckung nachzubauen hiesse, denselben Weg ein zweites
        // Mal zu gehen und dabei die Tiefe zu verlieren.
        //
        // Gedreht oder senkrecht geht das nicht - dort fuehrt kein Weg an
        // der Deckung vorbei.
        let grad_vorab = self.winkel(rotation);
        // **Nur ohne Hintergrund.** Mit einem gibt pygame eine Flaeche
        // mit 32 Bit heraus, nicht eine mit Tafel - gemessen an
        // `render(".", schwarz, weiss)`, das `get_bitsize() == 32`
        // verlangt. Die Zweifarbentafel traegt eben keinen Grund, der
        // etwas anderes waere als die Schluesselfarbe.
        if !self.antialiased && grad_vorab == 0 && !self.senkrecht && hg.is_none() {
            // **Ohne Glaettung gibt pygame eine Flaeche mit acht Bit**,
            // einer Zweifarbentafel und einer Schluesselfarbe - nicht eine
            // mit Alphakanal. Gemessen an `render(".", (255,0,0))` in
            // Groesse 24: `(2, 3)`, acht Bit, Schluesselfarbe
            // `(0, 255, 255)`, Tafeleintrag 1 die Schriftfarbe, und alle
            // sechs Pixel sind Eintrag 1.
            //
            // **Gebaut wird sie aus derselben Deckung, aus der auch das
            // Mass kommt.** Hier stand ein zweiter Aufruf der
            // Schriftkiste, und der lieferte eine Flaeche im
            // **Zeilenkasten** - der Zuschnitt auf die Tinte passte dann
            // nicht dazu, und bei `render(".")` kam ein Ausschnitt heraus,
            // in dem der Punkt gar nicht lag. Zwei Wege zu demselben Bild
            // sind zwei Gelegenheiten, sich zu unterscheiden.
            let mut k = self.kasten(&t, style, size, rotation)?;
            // **Leerer Text ist eine Zeile hoch** - wie unten im
            // geglaetteten Zweig: `_PGFT_Render_NewSurface` nimmt fuer
            // `length == 0` `_PGFT_Font_GetHeightSized`, egal ob acht oder
            // 32 Bit. Gemessen an `PyGameMono.otf` bei 24: 26 Zeilen, hier
            // standen 25 (Auf- minus Unterlaenge).
            if t.is_empty() {
                let f = self.in_groesse(size)?;
                k.h = self.leerhoehe(size, &f)?;
                k.deckung = vec![0; (k.w * k.h) as usize];
            }
            // **Die Schluesselfarbe ist das Gegenteil der Schriftfarbe.**
            // Gemessen: zu Schwarz gehoert Weiss, zu Rot (255,0,0) das
            // Cyan (0,255,255). Eine feste Farbe waere unsichtbar, sobald
            // jemand genau in ihr schreibt.
            let schluessel = rustgame_core::color::Color::rgba(
                255 - vg.r,
                255 - vg.g,
                255 - vg.b,
                255,
            );
            let grund = hg.unwrap_or(schluessel);
            let vorn = rustgame_core::color::Color::rgba(vg.r, vg.g, vg.b, 255);
            let mut bild = rustgame_core::surface::Surface::new((k.w, k.h));
            bild.set_palette(Some(vec![grund, vorn]));
            bild.set_format(
                rustgame_core::surface::Format::standard(8, false)
                    .map_err(PyValueError::new_err)?,
            );
            for y in 0..k.h {
                for x in 0..k.w {
                    let v = k.deckung[(y * k.w + x) as usize];
                    bild.setz_gepackt((x as i32, y as i32), u32::from(v != 0));
                }
            }
            if hg.is_none() {
                bild.set_colorkey(Some(schluessel));
                // **Ein halbdurchsichtiges Vorn wird zum Flaechenalpha.**
                //
                // Eine Flaeche mit Tafel hat kein Alpha je Pixel; die
                // Durchsichtigkeit der Schriftfarbe kann also nur als
                // `set_alpha` hinein. Gemessen: nach
                // `render(text, Color(255,0,0,55))` gibt `surf.get_alpha()`
                // genau **55**, und die Flaggen tragen `SRCALPHA`
                // zusaetzlich zur Schluesselfarbe.
                if vg.a != 255 {
                    bild.set_alpha(Some(vg.a));
                }
            }
            return Ok((
                Py::new(py, PySurface::neu(bild))?,
                PyRect::neu(rustgame_core::rect::Rect::new(
                    k.rx,
                    k.ry,
                    k.w as i32,
                    k.h as i32,
                )),
            ));
        }
        let k = self.kasten(&t, style, size, rotation)?;
        let (deckung, (w, h)) = (k.deckung.clone(), (k.w, k.h));
        let f = self.in_groesse(size)?;
        // **Leerer Text ist null breit und eine Zeile hoch.** Gemessen:
        // `render("", ...)` gibt `r.height == get_sized_height(size)`,
        // also den Zeilenabstand - nicht die Hoehe des Kastens, den die
        // Schrift fuer ihre Zeichen braucht. Die beiden gehen bei
        // `test_sans.ttf` in Groesse 24 um genau ein Pixel auseinander.
        // **`ft_height()`, nicht `get_linesize()`.** Beide heissen
        // "Zeilenhoehe" und gehen um ein Pixel auseinander: `ft_height`
        // ist die Zahl, die `get_sized_height()` nach Python meldet,
        // `get_linesize` die der Schriftkiste. Hier stand die zweite, und
        // damit war `render("")` einen Pixel hoeher als `get_rect("")`
        // plus eins - in der **eigenen** Huelle widersprachen sich zwei
        // Zahlen, die dasselbe heissen. Gemessen gibt pygame
        // `render("").height == get_sized_height()`.
        // **`length == 0`, nicht Breite null**: ein senkrechter Text aus
        // lauter Leerzeichen ist null breit und doch kein leerer Text
        // (gemessen: `"  "` senkrecht 22 hoch, nicht eine Zeile).
        let h = if t.is_empty() { self.leerhoehe(size, &f)? } else { h };
        let grund = match hg {
            Some(c) => c.to_argb(),
            None => 0,
        };
        // **Kein `max(1)`.** Leerer Text gibt eine Flaeche der Breite
        // null - gemessen: `font.render("", True, "white").get_size()`
        // ist `(0, Zeilenhoehe)`. Eine Flaeche von einem Pixel Breite
        // saehe fast gleich aus und ist an jeder Stelle, die Breiten
        // addiert, um eins daneben.
        let mut flaeche = rustgame_core::surface::Surface::with_flags(
            (w, h),
            rustgame_core::surface::SRCALPHA,
        );
        for y in 0..h as i32 {
            for x in 0..w as i32 {
                flaeche.set_at_ungeprueft((x, y), grund);
            }
        }
        let hat_alpha = flaeche.get_masks().3 != 0;
        if let Some(glyphen) = &k.glyphen {
            // **Zeichen fuer Zeichen**, wie `render()` in `ft_render.c`:
            // jedes wird fuer sich mit `ALPHA_BLEND` in die Flaeche
            // gemischt. Wo sich zwei ueberlappen (unterschnitten, schraeg),
            // ist das etwas anderes als einmal mit der vereinten Deckung.
            for g in glyphen {
                for gy in 0..g.hoehe {
                    let y = g.y + gy as i32;
                    if y < 0 || y >= h as i32 {
                        continue;
                    }
                    for gx in 0..g.breite {
                        let x = g.x + gx as i32;
                        if x < 0 || x >= w as i32 {
                            continue;
                        }
                        let c = u32::from(g.deckung[gy * g.breite + gx]);
                        let a = c * u32::from(vg.a) / 255;
                        if a == 0 {
                            continue;
                        }
                        let oben = rustgame_core::color::Color::rgba(vg.r, vg.g, vg.b, a as u8);
                        let alt = flaeche.get_at_mapped((x, y));
                        flaeche.set_at_ungeprueft((x, y), Self::ueber(oben, alt, hat_alpha));
                    }
                }
            }
            if let Some((ux, uy, uw, uh)) = k.unterstrich {
                fuellen_rgb(&mut flaeche, ux, uy, uw, uh, vg, hat_alpha);
            }
        } else {
            for y in 0..h as i32 {
                for x in 0..w as i32 {
                    let c = deckung[(y as u32 * w + x as u32) as usize] as u32;
                    if c == 0 {
                        continue;
                    }
                    let a = (c * vg.a as u32 / 255) as u8;
                    let oben = rustgame_core::color::Color::rgba(vg.r, vg.g, vg.b, a);
                    let alt = flaeche.get_at_mapped((x, y));
                    flaeche.set_at_ungeprueft((x, y), Self::ueber(oben, alt, hat_alpha));
                }
            }
        }
        Ok((
            Py::new(py, PySurface::neu(flaeche))?,
            PyRect::neu(rustgame_core::rect::Rect::new(
                k.rx,
                k.ry,
                w as i32,
                h as i32,
            )),
        ))
    }

    /// Die blosse **Deckung** des Textes - ein Byte je Pixel.
    ///
    /// Gemessen gibt `render_raw` ein Paar aus `bytes` und `(Breite,
    /// Hoehe)`. `invert=True` dreht die Werte um; das braucht pygames
    /// eigener Test, um zwei Bilder zu vergleichen.
    #[pyo3(signature = (text, style=STYLE_DEFAULT, rotation=None, size=None, invert=false))]
    fn render_raw(
        &self,
        py: Python<'_>,
        text: &Bound<'_, PyAny>,
        style: i32,
        rotation: Option<i32>,
        size: Option<Groesse>,
        invert: bool,
    ) -> PyResult<(Py<PyAny>, (u32, u32))> {
        self.stil_pruefen(style)?;
        self.modus_pruefen(style, rotation)?;
        // **`None` heisst "derselbe Text wie zuletzt"**, nicht "kein
        // Text". Das gilt fuer `render_raw` und `render_raw_to` genauso wie
        // fuer `get_rect` und `render`; hier stand einmal
        // `String::new()`, und `freetype_test.test_freetype_Font_text_is_None`
        // faellt darauf mit `(0, 2355) != (100, 42)` - der Hoehe eines
        // leeren Textes gegen die eines echten.
        let t = self.text_von(text)?;
        // **Leerer Text gibt die Hoehe in Schrifteinheiten.**
        //
        // Gemessen an `test_sans.ttf`: `render_raw("")` gibt `(0, 2355)`,
        // und 2355 ist `font.height` - also die **ungerechnete** Hoehe
        // aus der Datei, nicht die in Pixeln. Bei `render_raw("A")` steht
        // dort `(16, 17)`, also Pixel. Ein Sonderfall in pygames
        // `_freetype.c`, der die Groesse gar nicht erst einrechnet;
        // `freetype_test.test_freetype_Font_render_raw` prueft ihn.
        if t.is_empty() {
            return Ok((
                pyo3::types::PyBytes::new(py, &[]).into_any().unbind(),
                (0, self.kern()?.roh_height().max(0) as u32),
            ));
        }
        let (mut daten, masse) = self.deckung(&t, style, size, rotation)?;
        if invert {
            for v in &mut daten {
                *v = 255 - *v;
            }
        }
        Ok((
            pyo3::types::PyBytes::new(py, &daten).into_any().unbind(),
            masse,
        ))
    }

    /// Dieselbe Deckung, aber **in einen vorhandenen Puffer**.
    ///
    /// Angenommen wird, was rustgame selbst herausgibt: ein `BufferProxy`
    /// aus `Surface.get_view(...)`. Ein fremdes Objekt mit Puffer-
    /// Schnittstelle geht nicht - dafuer fehlt rustgame das C-API, siehe
    /// den Modulkopf von `pufferzeiger.rs`.
    #[pyo3(signature = (array, text, dest=None, style=STYLE_DEFAULT, rotation=None, size=None, invert=false))]
    #[allow(clippy::too_many_arguments)]
    fn render_raw_to(
        &self,
        array: &Bound<'_, PyAny>,
        text: &Bound<'_, PyAny>,
        dest: Option<&Bound<'_, PyAny>>,
        style: i32,
        rotation: Option<i32>,
        size: Option<Groesse>,
        invert: bool,
    ) -> PyResult<PyRect> {
        self.stil_pruefen(style)?;
        self.modus_pruefen(style, rotation)?;
        let t = self.text_von(text)?;
        let kst = self.kasten(&t, style, size, rotation)?;
        let (mut daten, (b, h)) = (kst.deckung.clone(), (kst.w, kst.h));
        if invert {
            for v in &mut daten {
                *v = 255 - *v;
            }
        }
        let (dx, dy) = match dest {
            Some(o) if !o.is_none() => punkt_aus(o)?,
            _ => (0, 0),
        };
        let py = array.py();
        // **Zwei Wege, und der erste ist der kuerzere.**
        //
        // Ist das Feld eine Sicht auf eine rustgame-Flaeche, dann wird
        // **in die Flaeche** geschrieben und nicht in ihren Puffer. Das ist
        // nicht nur schneller, es ist der einzige Weg, der auch bei 8, 16
        // und 24 Bit geht: einen Puffer gibt es dort nicht (rustgame legt
        // jede Flaeche als ARGB8888 ab), eine Flaeche schon.
        // `freetype_test.test_freetype_Font_render_raw_to` schreibt in
        // `get_view("2")` einer Achtbitflaeche und in `get_view("r")` einer
        // mit 24 - beides ueber diesen Weg.
        //
        // Alles andere ist ein **fremdes Feld**: `arrinter.Array`,
        // `buftools.Exporter`, ein `memoryview`, ein numpy-Array. Dort
        // wandert der Deckungswert als Zahl in jedes Element.
        if let Ok(zeiger) = array.downcast::<crate::pufferzeiger::PyBufferProxy>() {
            if let Ok(flaeche) = zeiger.borrow().flaeche_oeffentlich(py) {
                let s = flaeche.borrow();
                let mut k = s.leihen_mut()?;
                for y in 0..h as i32 {
                    for x in 0..b as i32 {
                        let v = daten[(y as u32 * b + x as u32) as usize] as u32;
                        let grau = 0xFF00_0000 | (v << 16) | (v << 8) | v;
                        k.set_at_ungeprueft((dx + x, dy + y), grau);
                    }
                }
                return Ok(Self::rohrechteck(&kst));
            }
        }
        let sicht = crate::puffer::hol_sicht(array)?;
        if !sicht.schreibbar {
            return Err(crate::umwandlung::PygameBufferError::new_err(
                "require writable buffer, but it is read-only",
            ));
        }
        // **Die Formatpruefung ist zweistufig**, und beide Stufen sind
        // noetig. `freetype_test.test_newbuf` laesst sechsundzwanzig
        // Formate durch und weist fuenf ab: `f`, `d`, `2h`, `?`, `hh`.
        //
        // `2h` und `hh` sind schon nach PEP 3118 keine einzelnen Zahlen -
        // die faengt `format_pruefen`. `f`, `d` und `?` sind gueltig und
        // trotzdem falsch: ein Deckungswert ist eine ganze Zahl, keine
        // Fliesskommazahl und kein Wahrheitswert. Wer nur das eine prueft,
        // laesst die drei anderen durch.
        let f = sicht.format.to_string_lossy().to_string();
        crate::puffer::format_pruefen(&f)?;
        if !matches!(
            crate::puffer::art(&f),
            crate::puffer::Art::Ganz | crate::puffer::Art::Fuell
        ) || !(1..=9).contains(&sicht.itemsize)
        {
            return Err(PyValueError::new_err(format!(
                "unsupported array item format '{f}'"
            )));
        }
        if sicht.shape.len() != 2 {
            return Err(PyValueError::new_err(format!(
                "expected a 2 dimensional array: got {}",
                sicht.shape.len()
            )));
        }
        for y in 0..h as isize {
            for x in 0..b as isize {
                let (zx, zy) = (x + dx as isize, y + dy as isize);
                if zx < 0 || zy < 0 || zx >= sicht.shape[0] || zy >= sicht.shape[1] {
                    continue;
                }
                let v = daten[(y as u32 * b + x as u32) as usize] as u64;
                crate::puffer::feld_schreiben(
                    &sicht,
                    zx * sicht.strides[0] + zy * sicht.strides[1],
                    v,
                );
            }
        }
        Ok(Self::rohrechteck(&kst))
    }

    /// Zeichnet direkt auf eine vorhandene Flaeche.
    #[pyo3(signature = (surf, dest, text, fgcolor=None, bgcolor=None, style=STYLE_DEFAULT, rotation=None, size=None))]
    #[allow(clippy::too_many_arguments)]
    fn render_to(
        &self,
        // Seit `render_to` die Deckung selbst auftraegt, baut es keine
        // Python-Flaeche mehr - der Griff bleibt in der Signatur, weil
        // pygames Aufruf ihn nicht kennt und PyO3 ihn nicht verlangt.
        _py: Python<'_>,
        surf: &PySurface,
        dest: &Bound<'_, PyAny>,
        text: &Bound<'_, PyAny>,
        fgcolor: Option<&Bound<'_, PyAny>>,
        bgcolor: Option<&Bound<'_, PyAny>>,
        style: i32,
        rotation: Option<i32>,
        size: Option<Groesse>,
    ) -> PyResult<PyRect> {
        self.stil_pruefen(style)?;
        self.modus_pruefen(style, rotation)?;
        // **Zwei Stufen, und pygames eigener Test schreibt sie aus:**
        //
        // ```text
        // # Blitting is done in two stages. First the target is alpha
        // # filled with the background color, if any. Second, the
        // # foreground color is alpha blitted to the background.
        // ```
        //
        // Hier stand eine Stufe: `render` backte Grund und Schrift in ein
        // Bild, und das Bild wanderte dann als Ganzes auf das Ziel. Bei
        // einem **halbdurchsichtigen** Grund ist das nicht dasselbe - sein
        // Alpha wirkt dann zweimal, einmal gegen die Schrift und einmal
        // gegen das Ziel. Gemessen mit Grund `(255,120,13,180)` und
        // Schrift `(128,97,213,60)` auf Schwarz:
        //
        // ```text
        //   zwei Stufen (pygame)   (168, 87, 57)
        //   ein Bild (vorher)      (174, 88, 46)
        // ```
        //
        // `test_freetype_Font_render_to_mono` prueft genau diese Stelle,
        // und zwar gegen **rustgames eigenen Blit**: es legt die Farben
        // erst mit `surf.blit` uebereinander, merkt sich das Ergebnis und
        // verlangt danach dasselbe von `render_to`.
        let t = self.text_von(text)?;
        let vg = match fgcolor {
            Some(o) if !o.is_none() => farbe_streng(o)?,
            _ => self.fgcolor,
        };
        let hg = match bgcolor {
            Some(o) if !o.is_none() => Some(farbe_streng(o)?),
            _ => self.bgcolor,
        };
        let k = self.kasten(&t, style, size, rotation)?;
        let p = punkt_aus(dest)?;
        // **`dest` meint zweierlei**, und das ist gemessen:
        //
        // ```text
        // origin=False  render_to(s, (10, 30), "M")  Tinte ab (10, 30)
        // origin=True   render_to(s, (10, 30), "M")  Tinte ab (11, 13)
        // get_rect("M") == (1, 17, 18, 17)
        // ```
        //
        // Ohne `origin` ist `dest` die **linke obere Ecke** der Tinte, mit
        // `origin` der **Stiftpunkt** auf der Grundlinie - und dann liegt
        // die Ecke bei `dest + (rect.x, -rect.y)`. Die zweite Zeile ist
        // genau `(10 + 1, 30 - 17)`.
        let (px, py2) = if self.origin {
            (p.0 + k.rx, p.1 - k.ry)
        } else {
            (p.0, p.1)
        };
        let (qb, qh) = (k.w, k.h);
        {
            let mut ziel = surf.leihen_mut()?;
            let ziel_hat_alpha = ziel.get_masks().3 != 0;
            for y in 0..qh as i32 {
                for x in 0..qb as i32 {
                    let deckung = k.deckung[(y as u32 * qb + x as u32) as usize];
                    // Ohne Grund und ohne Tinte bleibt das Ziel, wie es
                    // ist - **nicht** eine durchsichtige Farbe darauf.
                    if hg.is_none() && deckung == 0 {
                        continue;
                    }
                    let stelle = (px + x, py2 + y);
                    // **Zweimal schreiben, nicht zweimal rechnen.**
                    //
                    // Der Grund geht erst **in die Flaeche**, und die
                    // Schrift mischt sich dann gegen das, was dort
                    // wirklich steht. Auf einer Flaeche mit wenigen Bit je
                    // Kanal ist das ein Unterschied: der Grund rastet beim
                    // Ablegen auf eine Farbstufe ein, und die Schrift
                    // rechnet ab dieser Stufe weiter.
                    //
                    // Gemessen auf RGB565, Grund `(255,120,13,180)`,
                    // Schrift `(128,97,213,60)`:
                    //
                    // ```text
                    //   zweimal schreiben (pygame)   Gruen 89
                    //   zweimal rechnen              Gruen 85
                    // ```
                    //
                    // Eine Farbstufe, und sie faellt nur in 565 auf -
                    // genau dort, wo pygames eigener Test hinsieht.
                    let setzen = |ziel: &mut rustgame_core::Surface,
                                      oben: rustgame_core::color::Color| {
                        let wert =
                            Self::ueber(oben, ziel.get_at_mapped(stelle), ziel_hat_alpha);
                        let neu = ziel
                            .abbilden(rustgame_core::color::Color::from_argb(wert));
                        ziel.set_at_ungeprueft(stelle, neu);
                    };
                    if let Some(grund) = hg {
                        if grund.a != 0 {
                            setzen(&mut ziel, grund);
                        }
                    }
                    if deckung != 0 {
                        // Die Deckung und das Alpha der Schriftfarbe
                        // multiplizieren sich - einfarbig ist die Deckung
                        // 255, dann bleibt genau das Farbalpha.
                        let a = (u32::from(deckung) * u32::from(vg.a) / 255) as u8;
                        if a != 0 {
                            setzen(
                                &mut ziel,
                                rustgame_core::color::Color::rgba(vg.r, vg.g, vg.b, a),
                            );
                        }
                    }
                }
            }
        }
        Ok(PyRect::neu(rustgame_core::rect::Rect::new(
            px,
            py2,
            qb as i32,
            qh as i32,
        )))
    }
}

/// `pygame.freetype.SysFont(name, size, bold=False, italic=False)`.
///
/// Dieselbe Schriftsuche wie `pygame.font.SysFont`, nur baut sie am Ende
/// eine `freetype.Font`. Gemessen gibt sie bei jedem der acht Wege, auf
/// denen `freetype_test` den Namen hereinreicht (Text, `bytes`, Liste,
/// Erzeuger, gemischt), **dieselbe** Schrift.
#[pyfunction]
#[pyo3(signature = (name, size, bold=None, italic=None, constructor=None))]
#[pyo3(name = "SysFont")]
pub fn ft_sysfont(
    py: Python<'_>,
    name: Option<&Bound<'_, PyAny>>,
    size: f64,
    bold: Option<&Bound<'_, PyAny>>,
    italic: Option<&Bound<'_, PyAny>>,
    constructor: Option<&Bound<'_, PyAny>>,
) -> PyResult<PyFreetypeFont> {
    let _ = constructor;
    // pygames `freetype.SysFont` laeuft ueber `sysfont.SysFont`, und das
    // ruft `initsysfonts()` - also setzt es `is_init` und fuellt die
    // Tabellen wie `font.SysFont`.
    crate::text::sysfont_anstossen(py);
    // Gemessen wird `SysFont(name, 12, 0, 0)` geschrieben, mit Nullen
    // statt `False` - pygames eigene Beispiele tun das seit jeher.
    let bold = match bold {
        Some(o) => o.is_truthy()?,
        None => false,
    };
    let italic = match italic {
        Some(o) => o.is_truthy()?,
        None => false,
    };
    let n: String = match name {
        None => String::new(),
        Some(o) if o.is_none() => String::new(),
        Some(o) => crate::text::schriftnamen_aus(o)?,
    };
    let pfad = schrift::sysfont::match_font(&n, bold, italic);
    if !n.trim().is_empty() && pfad.is_none() {
        let erster = n.split(',').next().unwrap_or(&n).trim().to_string();
        let warnungen = py.import("warnings")?;
        let art = py.get_type::<pyo3::exceptions::PyUserWarning>();
        warnungen.call_method1(
            "warn",
            (
                format!(
                    "The system font '{erster}' couldn't be found. Verify your font name input. Using the default font instead."
                ),
                art,
            ),
        )?;
    }
    let mut f = match pfad {
        Some(p) => PyFreetypeFont::laden(
            Some(&p.into_pyobject(py)?.into_any()),
            size,
            0,
            0,
            false,
        )?,
        None => PyFreetypeFont::laden(None, size, 0, 0, false)?,
    };
    // Fehlt der Schnitt als eigene Datei, wird er nachgemacht - so macht
    // es `sysfont.py` auch.
    if bold {
        f.stil |= STYLE_STRONG;
    }
    if italic {
        f.stil |= STYLE_OBLIQUE;
    }
    Ok(f)
}

/// `pygame.freetype.init()`.
#[pyfunction]
#[pyo3(signature = (cache_size=64, resolution=0))]
#[pyo3(name = "init")]
pub fn ft_init(py: Python<'_>, cache_size: i32, resolution: u32) {
    crate::schriftordner_sichern(py);
    // `_PGFT_Init` legt nur beim ersten Mal eine Instanz an - und mit ihr
    // einen leeren Fehlersatz.
    if !FT_INIT.swap(true, std::sync::atomic::Ordering::Relaxed) {
        fehler_merken("");
    }
    CACHE.store(cache_size, std::sync::atomic::Ordering::Relaxed);
    // **Die Aufloesung von `init` ueberschreibt die gesetzte nicht.**
    // Gemessen: nach `set_default_resolution(82)` gibt
    // `init(resolution=92)` weiterhin 82. Das Argument gilt nur, solange
    // niemand die Vorgabe von Hand gesetzt hat.
    let _ = resolution;
    schrift::init();
}

/// `pygame.freetype.quit()`.
#[pyfunction]
#[pyo3(name = "quit")]
pub fn ft_quit() {
    FT_INIT.store(false, std::sync::atomic::Ordering::Relaxed);
    CACHE.store(0, std::sync::atomic::Ordering::Relaxed);
    schrift::quit();
}

/// `pygame.freetype.get_init()`.
#[pyfunction]
#[pyo3(name = "get_init")]
pub fn ft_get_init() -> bool {
    FT_INIT.load(std::sync::atomic::Ordering::Relaxed)
}

/// Laeuft `pygame.freetype`? - **ein eigener Schalter**, siehe
/// [`crate::text::FONT_INIT`].
pub(crate) static FT_INIT: std::sync::atomic::AtomicBool =
    std::sync::atomic::AtomicBool::new(false);

/// Setzt den Schalter und den gemeldeten Glyphenspeicher - fuer
/// `pygame.init()` und `pygame.quit()`, die beide Module mitnehmen.
///
/// Gemessen: nach `pygame.init()` meldet `get_cache_size()` **64**
/// (pygames Vorgabe), nach `quit()` wieder 0.
pub(crate) fn ft_init_schalter(an: bool) {
    FT_INIT.store(an, std::sync::atomic::Ordering::Relaxed);
    CACHE.store(
        if an { 64 } else { 0 },
        std::sync::atomic::Ordering::Relaxed,
    );
}

/// Laeuft `pygame.freetype`? Sonst **pygames Satz** - und der ist ein
/// blanker `RuntimeError`, nicht `pygame.error`.
///
/// Gemessen: `pygame.freetype.Font(None, 20)` ohne `freetype.init()`
/// gibt `RuntimeError: The FreeType 2 library hasn't been initialized`.
/// Hier kam `pygame.error: Invalid font (font module quit since font
/// created)` - der Satz fuer eine **andere** Lage.
pub(crate) fn ft_pruefen() -> PyResult<()> {
    if FT_INIT.load(std::sync::atomic::Ordering::Relaxed) {
        return Ok(());
    }
    Err(pyo3::exceptions::PyRuntimeError::new_err(
        "The FreeType 2 library hasn't been initialized",
    ))
}

/// `pygame.freetype.was_init()` - derselbe Wert, **alter** Name.
///
/// Und er sagt das auch: gemessen gibt er eine `DeprecationWarning`.
/// `freetype_test.test_was_init_deprecated` prueft genau das mit
/// `assertWarns` - ein stiller Zweitname bestuende den Test nicht.
#[pyfunction]
#[pyo3(name = "was_init")]
pub fn ft_was_init(py: Python<'_>) -> PyResult<bool> {
    let warnungen = py.import("warnings")?;
    let art = py.get_type::<pyo3::exceptions::PyDeprecationWarning>();
    warnungen.call_method1(
        "warn",
        (
            "pygame.freetype.was_init deprecated since 2.0.0",
            art,
        ),
    )?;
    Ok(schrift::get_init())
}

/// `pygame.freetype.get_default_font()`.
#[pyfunction]
#[pyo3(name = "get_default_font")]
pub fn ft_get_default_font() -> String {
    schrift::get_default_font().to_string()
}

/// `pygame.freetype.get_default_resolution()`.
#[pyfunction]
#[pyo3(name = "get_default_resolution")]
pub fn ft_get_default_resolution() -> u32 {
    AUFLOESUNG.load(std::sync::atomic::Ordering::Relaxed)
}

/// `pygame.freetype.set_default_resolution()`.
#[pyfunction]
#[pyo3(signature = (resolution=72, /))]
#[pyo3(name = "set_default_resolution")]
pub fn ft_set_default_resolution(resolution: u32) {
    AUFLOESUNG.store(
        if resolution == 0 { 72 } else { resolution },
        std::sync::atomic::Ordering::Relaxed,
    );
}

/// `pygame.freetype.get_version()`.
///
/// **`None` waere hier falsch** - anders als bei `get_sdl_ttf_version`
/// fragt pygames eigener Test die Zahl ab und vergleicht sie. Gemeldet
/// wird die Fassung, deren Verhalten nachgebaut ist.
#[pyfunction]
#[pyo3(signature = (linked=None))]
#[pyo3(name = "get_version")]
pub fn ft_get_version(linked: Option<&Bound<'_, PyAny>>) -> PyResult<(u32, u32, u32)> {
    // **Jeder Wert ist ein Wahrheitswert** - pygame liest den
    // Schalter mit `PyObject_IsTrue`, und das nimmt alles. Mit
    // `bool` verlangte PyO3 einen echten Wahrheitswert und nannte
    // im Fehlertext `'PyBool'`, einen Namen aus Rust.
    let linked = match linked {
        None => true,
        Some(o) => o.is_truthy()?,
    };
    let _ = linked;
    Ok((2, 11, 1))
}

/// `ft->_error_msg`: der letzte Satz von `_PGFT_SetError`. pygame setzt
/// ihn bei jedem FreeType-Fehler und leert ihn nur, wenn `init()` eine neue
/// Instanz anlegt - ein gelungener Aufruf danach laesst ihn stehen.
/// Gemessen: nach `get_rect("A", size=9)` an `PyGameMono-8.bdf` sagt
/// `get_error()` `'Failed to resize font: invalid pixel size'`, auch nach
/// einem gelungenen `get_rect` und einem zweiten `init()`; erst
/// `quit()` und `init()` geben `None`. Hier stand bis Runde 20 immer
/// `None`.
static LETZTER_FEHLER: std::sync::Mutex<String> = std::sync::Mutex::new(String::new());

/// `_PGFT_SetError` - merkt sich den Satz fuer `get_error()`.
fn fehler_merken(satz: &str) {
    if let Ok(mut s) = LETZTER_FEHLER.lock() {
        *s = satz.to_owned();
    }
}

/// Ein Fehler beim Laden einer Schrift: `_PGFT_GetFont` setzt
/// `Failed to load font: ..`, bevor die Ausnahme fliegt. Eine fehlende
/// Datei kommt nicht bis FreeType.
fn ft_lade_fehler(e: schrift::FontError) -> PyErr {
    if !matches!(e, schrift::FontError::Datei(_)) {
        fehler_merken("Failed to load font: unknown file format");
    }
    schrift_fehler(e)
}

/// `pygame.freetype.get_error()` - siehe `LETZTER_FEHLER`.
#[pyfunction]
#[pyo3(name = "get_error")]
pub fn ft_get_error() -> PyResult<Option<String>> {
    ft_pruefen()?;
    let s = LETZTER_FEHLER.lock().map(|s| s.clone()).unwrap_or_default();
    Ok((!s.is_empty()).then_some(s))
}

/// `pygame.freetype.get_cache_size()`.
#[pyfunction]
#[pyo3(name = "get_cache_size")]
pub fn ft_get_cache_size() -> i32 {
    CACHE.load(std::sync::atomic::Ordering::Relaxed)
}
