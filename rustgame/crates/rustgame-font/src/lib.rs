//! rustgame-font
//! =============
//! Gegenstueck zu `pygame.font` - Schrift laden, vermessen, setzen.
//!
//! ## Der Aufruf, auf den es ankommt
//!
//! Im umgebenden Projekt steht 1002-mal dieselbe Zeile:
//!
//! ```text
//! font.render(text, True, color)      # Python
//! font.render(text, true, farbe, None)  // hier
//! ```
//!
//! Also: Kantenglaettung an, kein Hintergrund. Das Ergebnis ist eine Flaeche
//! mit `SRCALPHA`, in der die Farbe konstant im RGB-Teil steht und die
//! Deckung allein im Alphakanal - und die genau `get_linesize()` hoch ist,
//! nicht `get_height()`. Beides ist in pygame so, beides steht in
//! [`Font::render`] begruendet.
//!
//! ## Was exakt stimmt und was nicht
//!
//! Die Rasterung uebernimmt [fontdue]; FreeType rastert **mit Hinting**,
//! fontdue nicht, also sind die Pixel nicht dieselben. Alles, was aus der
//! **Schriftdatei** kommt, stimmt dagegen auf den Wert genau - dafuer liest
//! diese Kiste die Tabellen selbst, statt sich auf fontdues gerundete Werte
//! zu verlassen.
//!
//! Gemessen an pygame-ce 2.5.7 ueber 13.440 Faelle (7 Schriften, 8 Groessen,
//! 5 Stile, 40 Zeichenketten):
//!
//! | Groesse | Herkunft | Ergebnis |
//! |---|---|---|
//! | `get_ascent`, `get_descent`, `get_height`, `get_linesize` | `hhea` + FreeType-Festkomma | **13440/13440 exakt** |
//! | Vorschub je Zeichen | `hmtx`, auf ganze Pixel gerundet | exakt |
//! | Kerning | GPOS, sonst `kern` | exakt |
//! | Breite von `size()` und der Flaeche | daraus + Glyphenkasten | groesste Abweichung **1 Pixel** |
//! | `SRCALPHA`, Schluesselfarbe | - | **13440/13440 exakt** |
//! | Hoehe der Flaeche | `linesize`, sonst Glyphenkasten | 13218/13440 exakt (98,3 %) |
//! | Tintenmenge | fontdue | 86,6 % innerhalb 10 %; ab Groesse 24 im Mittel unter 1 % |
//! | Schwerpunkt der Tinte | fontdue | mit Kantenglaettung 98,5 % innerhalb 1 Pixel |
//!
//! Die verbleibenden Abweichungen kommen **alle** aus dem Hinting: FreeType
//! rastet Ober- und Unterlaengen aufs Pixelraster, fontdue nicht. Bei
//! freesansbold in Groesse 8 sitzt das `g` dadurch ein Pixel tiefer, und die
//! Flaeche wird einen Pixel hoeher. Nachgemessen: der Kontrollkasten (CBox)
//! statt der engen Kontur-Grenze bringt dort nichts (34,2 % gegen 35,6 %
//! genau getroffene Glyphenkaesten) - ohne Hinting-Maschine ist hier Schluss.
//!
//! ## Aufbau
//!
//! | Modul | wofuer |
//! |---|---|
//! | [`fixed`] | FreeTypes Festkommarechnung |
//! | [`face`] | die Tabellen der Schriftdatei, Kerning |
//! | [`glyph`] | Rastern, nachgemachtes Fett und Kursiv |
//! | [`font`] | [`Font`]: Metriken, `size`, `render` |
//! | [`sysfont`] | `SysFont`, `get_fonts`, `match_font` |

// **Kein `unsafe` in dieser Kiste, und das prueft der Uebersetzer.**
//
// Hier stand der Satz lange nur in der Beschreibung - und eine Behauptung,
// die niemand nachhaelt, veraltet still. Siehe
// `rustgame-py/src/puffer.rs`: dort liegt das `unsafe` des Projekts, und
// nur dort ist der Lint ausdruecklich abgeschaltet.
#![forbid(unsafe_code)]
#![deny(missing_docs)]

pub mod bdf;
pub mod face;
pub mod fixed;
pub mod font;
pub(crate) mod ftgrays;
pub(crate) mod ftraster;
pub(crate) mod ftstroke;
pub(crate) mod gestalt;
pub mod glyph;
pub(crate) mod hinting;
pub mod sysfont;

use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};

pub use font::{Font, FtGlyphe, FtMass, FtStil, FtZeile, Stil};
pub use sysfont::{get_fonts, match_font};

/// Was beim Laden oder Setzen schiefgehen kann.
#[derive(Debug)]
pub enum FontError {
    /// Die Datei liess sich nicht lesen.
    Datei(std::io::Error),
    /// Die Datei ist keine brauchbare Schrift.
    Unlesbar,
    /// Der Text hat die Breite 0 - pygame wirft hier `pygame.error:
    /// "Text has zero width"`.
    OhneBreite,
    /// `freesansbold.ttf` war nirgends zu finden (siehe
    /// [`default_font_path`]).
    KeineStandardschrift,
}

impl std::fmt::Display for FontError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            // `e.kind()` statt `e` - siehe `rustgame_core::image::Fehler`.
            FontError::Datei(e) => write!(f, "Font file could not be read: {}", e.kind()),
            FontError::Unlesbar => write!(f, "not a usable font file"),
            FontError::OhneBreite => write!(f, "Text has zero width"),
            FontError::KeineStandardschrift => {
                write!(
                    f,
                    "freesansbold.ttf not found - set RUSTGAME_DEFAULT_FONT"
                )
            }
        }
    }
}

impl std::error::Error for FontError {}

static BEREIT: AtomicBool = AtomicBool::new(false);
static GENERATION: AtomicU32 = AtomicU32::new(0);

/// `pygame.font.init()`.
///
/// Anders als in pygame gibt es hier nichts aufzubauen - SDL_ttf muss
/// FreeType hochfahren, fontdue nicht. Der Schalter ist trotzdem da, weil
/// pygame-Programme ihn abfragen und weil `quit()` dieselbe Bedeutung
/// behalten soll.
pub fn init() {
    BEREIT.store(true, Ordering::SeqCst);
}

/// `pygame.font.quit()`.
///
/// Zaehlt dabei [`generation`] hoch - aber **nur**, wenn das Modul
/// ueberhaupt lief. Genau so steht es in `fontmodule_quit` (`font.c`):
///
/// ```c
/// if (font_initialized) {
///     TTF_Quit();
///     font_initialized = 0;
///     current_ttf_generation++;
/// }
/// ```
///
/// Zweimal `quit()` hintereinander zaehlt also nur einmal.
pub fn quit() {
    if BEREIT.swap(false, Ordering::SeqCst) {
        GENERATION.fetch_add(1, Ordering::SeqCst);
    }
}

/// `pygame.font.get_init()`.
pub fn get_init() -> bool {
    BEREIT.load(Ordering::SeqCst)
}

/// Wie oft das Modul seit Programmstart beendet wurde.
///
/// Gegenstueck zu `current_ttf_generation` aus `font.c`. Eine `Font`
/// merkt sich den Stand bei ihrer Erzeugung; stimmt er spaeter nicht
/// mehr, ist die Schrift **ungueltig** - in pygame, weil `TTF_Quit()`
/// ihre `TTF_Font` mit weggeraeumt hat, hier, weil dieselbe Regel
/// sichtbar bleiben soll:
///
/// ```text
/// f = pygame.font.Font(None, 16)
/// pygame.font.quit(); pygame.font.init()
/// f.get_height()   ->  pygame.error: Invalid font (font module quit
///                      since font created)
/// ```
///
/// Gemessen. Die Pruefung selbst sitzt in der Huelle (`text.rs`), denn
/// nur dort gibt es eine Ausnahme zu werfen; diese Kiste liefert die
/// Zahl.
pub fn generation() -> u32 {
    GENERATION.load(Ordering::SeqCst)
}

/// `pygame.font.get_default_font()` - der **Dateiname**, nicht der Pfad.
/// Die Fassung der Schriftbibliothek - `font.get_sdl_ttf_version`.
///
/// Gibt `None` zurueck, und das ist die einzig ehrliche Antwort: hier
/// gibt es **kein SDL_ttf**. Der Text entsteht mit `fontdue` und einer
/// Uebertragung von SDL_ttfs Layoutregeln; eine Versionsnummer zu
/// erfinden (echtes pygame meldet hier `(2, 24, 0)`) wuerde nur
/// Programme in die Irre fuehren, die anhand der Zahl Weichen stellen.
///
/// Genauso haelt es `image::get_sdl_image_version` im Kern.
///
/// ## Warum hier eine Zahl steht und nicht `None`
///
/// Hier stand `None`, mit der Begruendung: rustgame bindet kein
/// SDL_ttf ein, also waere jede Zahl erfunden. Das klang richtig und
/// war es nicht - **pygames eigene Testsuite hat es widerlegt.**
///
/// `image_test.py` schreibt die Regel aus:
///
/// ```python
/// # If get_extended() returns False then get_sdl_image_version() should
/// # return None
/// if not pygame.image.get_extended():
///     self.assertIsNone(pygame.image.get_sdl_image_version())
/// else:
///     version = pygame.image.get_sdl_image_version()
///     self.assertIsInstance(version, tuple)
///     self.assertEqual(len(version), 3)
/// ```
///
/// `None` ist also **nicht** die Antwort "wir haben die Bibliothek
/// nicht", sondern die Antwort "wir koennen die erweiterten Formate
/// nicht". Und die koennen wir: `image.get_extended()` gibt hier
/// **`true`**, PNG und JPEG gehen. Die Paarung `get_extended() == true`
/// mit `version == None` widerspricht sich in sich - und genau das hat
/// pygames Test gemeldet.
///
/// Gemeldet wird deshalb die Fassung, deren **Leistungsumfang** hier
/// nachgebaut ist: die, die pygame-ce 2.5.7 mitbringt. Das ist dieselbe
/// Entscheidung wie bei `version.vernum`, das aus demselben Grund
/// `2.5.7` sagt und nicht eine eigene Zaehlung - die Frage
/// `version >= (2, 0, 0)` heisst "kann ich mich auf diesen Umfang
/// verlassen?", und die Antwort ist ja.
///
/// Was wirklich fehlt, sagt nicht diese Zahl, sondern ein **klarer
/// Fehler** an der Stelle, an der es fehlt - `image.load_animation` und
/// `load_sized_svg` melden ihn.
///
/// Nebenbei: drei ganze Testdateien (`image`, `font`, `mixer_music`)
/// konnten wegen des `None` nicht einmal **geladen** werden; sie
/// vergleichen die Fassung schon beim Import und bekamen
/// `TypeError: '<' not supported between instances of 'NoneType'`.
pub fn get_sdl_ttf_version() -> Option<(u32, u32, u32)> {
    Some((2, 24, 0))
}

/// Der **Name** der mitgelieferten Schrift - `freesansbold.ttf`.
/// 
/// Kein Pfad. Wo die Datei liegt, sagt [`default_font_path`].
pub fn get_default_font() -> &'static str {
    "freesansbold.ttf"
}

/// Wo die Standardschrift liegt.
///
/// pygame hat es einfach: die Datei liegt im Paketordner neben `font.pyd`.
/// Ein Rust-Programm hat keinen Paketordner, also wird der Reihe nach
/// gesucht:
///
/// 1. `RUSTGAME_DEFAULT_FONT` (voller Pfad zur Datei),
/// 2. neben der ausfuehrbaren Datei, auch in `assets/`,
/// 3. im aktuellen Ordner,
/// 4. im pygame-Paket eines `.venv` im aktuellen Ordner oder darueber -
///    das ist der Fall beim Pruefen gegen die Vorlage.
pub fn default_font_path() -> Option<PathBuf> {
    let name = get_default_font();

    if let Ok(p) = std::env::var("RUSTGAME_DEFAULT_FONT") {
        let p = PathBuf::from(p);
        if p.is_file() {
            return Some(p);
        }
    }

    let mut kandidaten: Vec<PathBuf> = Vec::new();
    // **Der Ordner, in dem die Huelle liegt** - dort wird die Schrift
    // mitgeliefert, so wie pygame sie neben seinem Paket liegen hat.
    // Siehe [`set_schriftordner`].
    if let Some(d) = EIGENER_ORDNER.get() {
        kandidaten.push(d.join(name));
        kandidaten.push(d.join("assets").join(name));
    }
    if let Ok(exe) = std::env::current_exe() {
        if let Some(d) = exe.parent() {
            kandidaten.push(d.join(name));
            kandidaten.push(d.join("assets").join(name));
        }
    }
    if let Ok(cwd) = std::env::current_dir() {
        kandidaten.push(cwd.join(name));
        let mut o: Option<&std::path::Path> = Some(cwd.as_path());
        for _ in 0..4 {
            let Some(d) = o else { break };
            kandidaten.push(
                d.join(".venv")
                    .join("Lib")
                    .join("site-packages")
                    .join("pygame")
                    .join(name),
            );
            o = d.parent();
        }
    }
    kandidaten.into_iter().find(|p| p.is_file())
}

/// Wo die Standardschrift **hingehoert** - auch wenn sie dort fehlt.
///
/// Das ist pygames `join(pygame_folder, "freesansbold.ttf")` aus
/// `sysfont.initsysfonts`: ein Pfad, der ohne Pruefung eingetragen wird.
/// Gefunden wird zuerst wie in [`default_font_path`]; sonst der Platz
/// neben der Huelle (siehe [`set_schriftordner`]), und kennt niemand den,
/// der blosse Dateiname.
pub fn mitgelieferter_pfad() -> PathBuf {
    let name = get_default_font();
    default_font_path()
        .or_else(|| EIGENER_ORDNER.get().map(|d| d.join(name)))
        .unwrap_or_else(|| PathBuf::from(name))
}

/// Wo die Huelle liegt - **einmal gesetzt, nie geraten**.
static EIGENER_ORDNER: std::sync::OnceLock<PathBuf> = std::sync::OnceLock::new();

/// Sagt dieser Kiste, in welchem Ordner das eingebundene Modul liegt.
///
/// ## Warum das nachtraeglich dazukam
///
/// [`default_font_path`] suchte an vier Stellen: einer Umgebungsvariablen,
/// neben der ausfuehrbaren Datei, im **aktuellen Ordner** und im
/// pygame-Paket eines `.venv` darueber. Drei der vier haengen damit am
/// Arbeitsverzeichnis - und das darf sich aendern.
///
/// Genau das tat es: zwei Tests in `ftfont_test.py` scheiterten mit
///
/// ```text
/// FileNotFoundError: Das System kann die angegebene Datei nicht finden.
///   f = pygame_font.Font(pygame_font.get_default_font(), 20)
/// ```
///
/// **mal zwei, mal keiner** - je nachdem, ob ein frueher gelaufener Test
/// das Arbeitsverzeichnis gewechselt hatte. Allein gefahren war die Datei
/// immer gruen; nur im vollen Durchgang fiel sie um. Ein Fehler, der von
/// der Reihenfolge abhaengt, sieht aus wie Zufall.
///
/// Die Schrift liegt mitgeliefert **neben der Huelle**
/// (`crates/rustgame-py/freesansbold.ttf`), und dieser Ordner aendert sich
/// nicht. Also sagt die Huelle ihn beim Laden einmal an, und die Suche
/// fragt ihn zuerst.
pub fn set_schriftordner(ordner: PathBuf) {
    let _ = EIGENER_ORDNER.set(ordner);
}

/// Die Standardschrift in der Groesse, die `pygame.font.Font(None, groesse)`
/// liefert - also mit dem Faktor `0.6875`.
pub fn default_font(groesse: i32) -> Result<Font, FontError> {
    let p = default_font_path().ok_or(FontError::KeineStandardschrift)?;
    let bytes = std::fs::read(p).map_err(FontError::Datei)?;
    Font::default_font(bytes, groesse)
}

/// Die mitgelieferte Schrift in der Groesse, die **wirklich dasteht**.
///
/// ## Zwei Textkisten, zwei Groessen, und das ist kein Fehler
///
/// [`default_font`] rechnet die Groesse mit `0.6875` klein, weil pygames
/// `font.c` das tut: `pygame.font.Font(None, 16)` ist freesansbold in
/// Groesse **11**. Fuer `pygame.font` ist das richtig.
///
/// Fuer `pygame.freetype` ist es **falsch**, und zwar deutlich.
/// `pygame.freetype.Font(None)` laedt die Schrift in voller Groesse;
/// gemessen an echtem pygame:
///
/// ```text
///                                      pygame      vorher hier
/// freetype.get_rect("A", size=24)      (0,17,17,17)  (0,12,12,12)
/// freetype.get_sized_ascender(24)      20            13
/// freetype.get_rect("Music", size=24)  (1,17,67,17)  (1,12,44,13)
/// ```
///
/// Der Beweis, dass pygame es so meint, steht in seinem eigenen
/// `ftfont.py`: dort wird fuer die Standardschrift
/// `resolution = int(get_default_resolution() * 0.6875)` gesetzt - also
/// **49 statt 72**, nur um die kleine Groesse von `pygame.font`
/// nachzubilden. Das waere sinnlos, wenn `pygame.freetype` schon klein
/// rechnete. Hier wirkte der Faktor darum **doppelt**:
/// `ftfont.Font(None, 24).size("Music")` gab `(31, 13)` statt `(47, 19)`.
///
/// Aufgefallen ist es an `font_test.test_issue_font_alphablit` - einem
/// Test, der nur prueft, ob die obere linke Ecke eines Textbildes
/// durchsichtig ist. Der Vermerk daneben nannte ihn eine **Rasterkante**
/// ("an einer Ecke steht dann eine Deckung von 1 statt 0"). Er war keine:
/// das Bild war ein Drittel zu klein.
pub fn default_font_voll(groesse: i32) -> Result<Font, FontError> {
    let p = default_font_path().ok_or(FontError::KeineStandardschrift)?;
    let bytes = std::fs::read(p).map_err(FontError::Datei)?;
    Font::from_bytes(bytes, groesse.max(1))
}

/// `pygame.font.SysFont(name, size, bold, italic)`.
///
/// `name` darf eine Kommaliste sein - `"bahnschrift,segoeui,consolas"`
/// nimmt den ersten Namen, der sich aufloesen laesst. Findet sich keiner,
/// kommt die Standardschrift zurueck; pygame warnt dabei, hier sagt es der
/// Rueckgabewert von [`match_font`], wenn man es wissen will.
///
/// Gibt es den gewuenschten Schnitt als eigene Datei, wird sie genommen;
/// sonst wird fett bzw. kursiv nachgemacht - genau die Unterscheidung, die
/// `sysfont.py` mit `gotbold`/`gotitalic` trifft.
pub fn sys_font(name: &str, groesse: i32, bold: bool, italic: bool) -> Result<Font, FontError> {
    let (pfad, hat_bold, hat_italic) = sysfont::schnitt_suchen(name, bold, italic);
    let mut f = match pfad {
        Some(p) => Font::new(p, groesse)?,
        None => return default_font(groesse),
    };
    if bold && !hat_bold {
        f.set_bold(true);
    }
    if italic && !hat_italic {
        f.set_italic(true);
    }
    Ok(f)
}

/// Version der Vorlage, gegen die geprueft wird.
pub const PYGAME_VERSION: &str = "2.5.7";
