//! sysfont.rs
//! ==========
//! `pygame/sysfont.py` (563 Zeilen) auf Rust uebertragen - die **Logik**,
//! nicht nur das Ergebnis.
//!
//! Unter Windows steht die Wahrheit ueber die installierten Schriften in der
//! Registry, nicht im Dateisystem: dort haengt an jedem Eintrag ein Name wie
//! `Arial Narrow Bold Italic (TrueType)`, und genau aus diesem Namen liest
//! pygame Fettung und Kursive heraus. Die Eigenheiten dieser Auswertung
//! stehen weiter unten bei [`eintrag_zerlegen`] - sie sind nicht huebsch,
//! aber sie entscheiden, welche Datei `match_font("arial")` liefert.
//!
//! Gelesen wird ueber `windows-registry`, also `RegQueryInfoKeyW` und
//! `RegEnumValueW` - genau die Schleife aus `winreg.QueryInfoKey` und
//! `winreg.EnumValue`, die `initsysfonts_win32` faehrt. Auf dieser
//! Reihenfolge beruht, welcher Eintrag gewinnt, wenn zwei Dateien auf
//! denselben Namen fallen.
//!
//! ## Warum nicht mehr `reg.exe query`
//!
//! Das war der erste Weg, ohne zusaetzliche Abhaengigkeit, und die
//! Reihenfolge stimmte. **Die Zeichen stimmten nicht:** `reg.exe` schreibt in
//! eine Pipe in der OEM-Codepage (hier 850), gelesen wurde als UTF-8. Jedes
//! Zeichen ausserhalb von ASCII wurde zum Ersatzzeichen, und das ist kein
//! Buchstabe - also fiel es in [`schlichter_name`] weg. Gemessen auf einem
//! deutschen Windows (`\u{e4}` ist das a mit Umlaut):
//!
//! ```text
//! pygame    lucidasanstypewriterschr\u{e4}g   lucidasanstypewriterfettschr\u{e4}g
//! rustgame  lucidasanstypewriterschrg        lucidasanstypewriterfettschrg
//! ```
//!
//! `get_fonts()` hatte bei pygame zwei Namen mit Umlaut, hier keinen.
//!
//! ## Die Falle mit `rstrip`
//!
//! `sysfont.py` schreibt `name.rstrip("(TrueType)")`. Das sieht aus wie
//! "Endung abschneiden", ist in Python aber "alle diese **Zeichen** hinten
//! abschneiden". Bei `Arial (TrueType)` kommt dasselbe heraus, bei anderen
//! Namen nicht unbedingt. Hier steht deshalb dieselbe Zeichenmengen-Logik
//! und nicht `strip_suffix`.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

/// Die Endungen, die pygame ueberhaupt betrachtet.
const ENDUNGEN: [&str; 3] = ["ttf", "ttc", "otf"];

/// Woerter, die pygame aus dem Namen wirft, bevor es Fett/Kursiv sucht.
const MODS: [&str; 6] = ["demibold", "narrow", "light", "unicode", "bt", "mt"];

/// Die Ersatzkette aus `create_aliases`, wortwoertlich uebernommen.
const ALIAS_GRUPPEN: &[&[&str]] = &[
    &[
        "monospace",
        "misc-fixed",
        "courier",
        "couriernew",
        "console",
        "fixed",
        "mono",
        "freemono",
        "bitstreamverasansmono",
        "verasansmono",
        "monotype",
        "lucidaconsole",
        "consolas",
        "dejavusansmono",
        "liberationmono",
    ],
    &[
        "sans",
        "arial",
        "helvetica",
        "swiss",
        "freesans",
        "bitstreamverasans",
        "verasans",
        "verdana",
        "tahoma",
        "calibri",
        "gillsans",
        "segoeui",
        "trebuchetms",
        "ubuntu",
        "dejavusans",
        "liberationsans",
    ],
    &[
        "serif",
        "times",
        "freeserif",
        "bitstreamveraserif",
        "roman",
        "timesroman",
        "timesnewroman",
        "dutch",
        "veraserif",
        "georgia",
        "cambria",
        "constantia",
        "dejavuserif",
        "liberationserif",
    ],
    &["wingdings", "wingbats"],
    &["comicsansms", "comicsans"],
];

/// Eine Schriftfamilie mit ihren Schnitten - **in der Reihenfolge des
/// Eintragens**, wie ein Python-`dict`.
///
/// Hier stand eine `HashMap`, und das war an zwei Stellen falsch:
///
/// * Wo pygame "irgendeinen" Schnitt nimmt (`list(styles.values())[0]` in
///   `match_font`, `list(styles.items())[0]` in `_load_single_font`), ist
///   das der **zuerst eingetragene**. Eine `HashMap` gibt ihre Eintraege
///   in einer Reihenfolge, die Rust **je Prozess neu wuerfelt** - dieselbe
///   Frage hatte also von Lauf zu Lauf eine andere Antwort.
/// * `sysfont.Sysfonts` zeigte die Schnitte sortiert, pygame in der
///   Reihenfolge der Registry: `{(False, False): .., (True, False): ..,
///   (True, True): .., (False, True): ..}` fuer Arial. Gleich nach `==`,
///   verschieden nach `repr` und beim Durchlaufen.
#[derive(Debug, Clone, Default)]
struct Schnitte(Vec<((bool, bool), PathBuf)>);

impl Schnitte {
    /// `d[k] = v`: ein Schluessel, den es schon gibt, **behaelt seinen
    /// Platz** und bekommt nur den neuen Pfad - so wie in Python.
    fn setzen(&mut self, stil: (bool, bool), pfad: PathBuf) {
        match self.0.iter_mut().find(|(s, _)| *s == stil) {
            Some(eintrag) => eintrag.1 = pfad,
            None => self.0.push((stil, pfad)),
        }
    }

    /// `d.get(k)`.
    fn get(&self, stil: &(bool, bool)) -> Option<&PathBuf> {
        self.0.iter().find(|(s, _)| s == stil).map(|(_, p)| p)
    }

    /// `list(d.items())[0]` - der zuerst eingetragene Schnitt.
    fn erster(&self) -> Option<&((bool, bool), PathBuf)> {
        self.0.first()
    }
}

/// Die einmal aufgebaute Schriftenliste.
#[derive(Debug, Default)]
pub struct SysTabelle {
    /// Namen in der Reihenfolge, in der sie zuerst aufgetaucht sind -
    /// `get_fonts()` gibt genau das zurueck.
    reihenfolge: Vec<String>,
    fonts: HashMap<String, Schnitte>,
    alias: HashMap<String, Schnitte>,
}

static TABELLE: OnceLock<SysTabelle> = OnceLock::new();

/// Baut die Liste einmal auf (`initsysfonts`).
pub fn tabelle() -> &'static SysTabelle {
    TABELLE.get_or_init(aufbauen)
}

fn aufbauen() -> SysTabelle {
    let mut t = SysTabelle::default();
    if cfg!(windows) {
        for (wurzel, pfad) in schluessel() {
            for (name, datei) in registry_werte(wurzel, pfad) {
                eintrag_aufnehmen(&mut t, &name, &datei);
            }
        }
    } else {
        for ordner in ["/usr/share/fonts", "/usr/local/share/fonts"] {
            ordner_durchsuchen(&mut t, Path::new(ordner));
        }
    }

    // pygame haengt die mitgelieferte Schrift ans Ende - und zwar als
    // *fetten* Schnitt, denn freesansbold ist einer. Das ist der Grund,
    // warum `match_font("freesansbold")` trotz `bold=False` etwas findet.
    //
    // **Immer, nicht nur wenn die Datei gefunden wird.** pygames
    // `initsysfonts` traegt `join(pygame_folder, "freesansbold.ttf")` ohne
    // Pruefung ein. Hier stand `if let Some(p) = default_font_path()`, und
    // die Suche hing am Arbeitsverzeichnis: aus `%TEMP%` gestartet gab
    // `get_fonts()` 390 statt 391 Namen und `match_font("freesansbold")`
    // `None` - gemessen im Modulvergleich, der dorthin wechselt.
    hinzufuegen(&mut t, "freesansbold", true, false, crate::mitgelieferter_pfad());

    aliase_bilden(&mut t);
    t
}

fn schluessel() -> Vec<(&'static str, &'static str)> {
    let mut v = Vec::new();
    for wurzel in ["HKLM", "HKCU"] {
        for pfad in [
            r"SOFTWARE\Microsoft\Windows NT\CurrentVersion\Fonts",
            r"SOFTWARE\Microsoft\Windows\CurrentVersion\Fonts",
        ] {
            v.push((wurzel, pfad));
        }
    }
    v
}

/// Liest die Werte eines Registry-Schluessels als (Name, Daten) - in der
/// Reihenfolge von `RegEnumValueW`, als UTF-16 und damit **mit Umlauten**.
///
/// Werte, die kein Text sind, fallen weg. pygame bekommt dort `bytes` oder
/// eine Zahl, und `splitext(font)[1].lower()` liegt dann nicht in
/// `OpenType_extensions` - der Eintrag wird ebenso uebersprungen.
#[cfg(windows)]
fn registry_werte(wurzel: &str, pfad: &str) -> Vec<(String, String)> {
    let basis = match wurzel {
        "HKLM" => windows_registry::LOCAL_MACHINE,
        _ => windows_registry::CURRENT_USER,
    };
    let Ok(schluessel) = basis.open(pfad) else {
        return Vec::new();
    };
    let Ok(werte) = schluessel.values() else {
        return Vec::new();
    };
    werte
        .filter_map(|(name, wert)| String::try_from(wert).ok().map(|d| (name, d)))
        .collect()
}

/// Ausserhalb von Windows gibt es keine Registry - dort wird gesucht.
#[cfg(not(windows))]
fn registry_werte(_wurzel: &str, _pfad: &str) -> Vec<(String, String)> {
    Vec::new()
}

/// Ein Registry-Eintrag, so wie `initsysfonts_win32` ihn behandelt.
fn eintrag_aufnehmen(t: &mut SysTabelle, name: &str, datei: &str) {
    let endung = Path::new(datei)
        .extension()
        .map(|e| e.to_string_lossy().to_lowercase())
        .unwrap_or_default();
    if !ENDUNGEN.contains(&endung.as_str()) {
        return;
    }
    let pfad = if Path::new(datei).parent().map(|p| p.as_os_str().is_empty()) != Some(false) {
        let windir = std::env::var("WINDIR").unwrap_or_else(|_| "C:\\Windows".to_string());
        Path::new(&windir).join("Fonts").join(datei)
    } else {
        PathBuf::from(datei)
    };

    // "Cambria & Cambria Math" traegt zwei Namen - beide zaehlen.
    for teil in name.split('&') {
        if !pfad.exists() {
            continue;
        }
        let (schlicht, bold, italic) = eintrag_zerlegen(teil);
        if schlicht.is_empty() {
            continue;
        }
        hinzufuegen(t, &schlicht, bold, italic, pfad.clone());
    }
}

/// `_parse_font_entry_win`: aus `Arial Narrow Bold Italic (TrueType)` wird
/// `("arial", true, true)`.
///
/// Dass `narrow` und `light` einfach wegfallen, ist kein Versehen von
/// pygame, sondern Absicht - und der Grund, warum `match_font("arial")` auf
/// diesem Rechner `ARIALN.TTF` liefert und nicht `arial.ttf`: Arial Narrow
/// steht in der Registry hinter Arial und ueberschreibt den Eintrag.
pub fn eintrag_zerlegen(name: &str) -> (String, bool, bool) {
    let mut n = name.to_string();
    if n.ends_with("(TrueType)") {
        // Pythons rstrip mit Zeichenmenge - siehe Modulkopf.
        n = n
            .trim_end_matches(|c: char| "(TrueType)".contains(c))
            .trim_end()
            .to_string();
    }
    let mut woerter: Vec<String> = n
        .to_lowercase()
        .split_whitespace()
        .map(String::from)
        .collect();
    for m in MODS {
        if let Some(i) = woerter.iter().position(|w| w == m) {
            woerter.remove(i);
        }
    }
    let mut bold = false;
    let mut italic = false;
    if let Some(i) = woerter.iter().position(|w| w == "bold") {
        woerter.remove(i);
        bold = true;
    }
    if let Some(i) = woerter.iter().position(|w| w == "italic") {
        woerter.remove(i);
        italic = true;
    }
    (schlichter_name(&woerter.concat()), bold, italic)
}

/// `_simplename`: nur Buchstaben und Ziffern, klein geschrieben.
pub fn schlichter_name(name: &str) -> String {
    name.chars()
        .filter(|c| c.is_alphanumeric())
        .flat_map(|c| c.to_lowercase())
        .collect()
}

fn hinzufuegen(t: &mut SysTabelle, name: &str, bold: bool, italic: bool, pfad: PathBuf) {
    if !t.fonts.contains_key(name) {
        t.reihenfolge.push(name.to_string());
    }
    // Wie in Python: der spaetere Eintrag gewinnt, der Platz bleibt.
    t.fonts
        .entry(name.to_string())
        .or_default()
        .setzen((bold, italic), pfad);
}

fn ordner_durchsuchen(t: &mut SysTabelle, ordner: &Path) {
    let Ok(eintraege) = std::fs::read_dir(ordner) else {
        return;
    };
    for e in eintraege.flatten() {
        let p = e.path();
        if p.is_dir() {
            ordner_durchsuchen(t, &p);
            continue;
        }
        let endung = p
            .extension()
            .map(|x| x.to_string_lossy().to_lowercase())
            .unwrap_or_default();
        if !ENDUNGEN.contains(&endung.as_str()) {
            continue;
        }
        let stamm = p
            .file_stem()
            .map(|x| x.to_string_lossy().to_string())
            .unwrap_or_default();
        let mut name = schlichter_name(&stamm);
        let mut bold = false;
        let mut italic = false;
        if name.contains("bold") {
            name = name.replace("bold", "");
            bold = true;
        }
        if name.contains("italic") {
            name = name.replace("italic", "");
            italic = true;
        }
        if !name.is_empty() {
            hinzufuegen(t, &name, bold, italic, p);
        }
    }
}

/// `create_aliases`: fehlende Sammelnamen auf vorhandene Schriften legen.
fn aliase_bilden(t: &mut SysTabelle) {
    for gruppe in ALIAS_GRUPPEN {
        let Some(gefunden) = gruppe.iter().find_map(|n| t.fonts.get(*n).cloned()) else {
            continue;
        };
        for n in gruppe.iter() {
            if !t.fonts.contains_key(*n) {
                t.alias.insert((*n).to_string(), gefunden.clone());
            }
        }
    }
}

impl SysTabelle {
    fn schnitte(&self, name: &str) -> Option<&Schnitte> {
        let n = schlichter_name(name);
        self.fonts.get(&n).or_else(|| self.alias.get(&n))
    }
}

/// Alle gefundenen Schriftnamen - Gegenstueck zu `pygame.font.get_fonts()`.
pub fn get_fonts() -> Vec<String> {
    tabelle().reihenfolge.clone()
}

/// Die ganze Tabelle, Name fuer Name - fuer `sysfont.Sysfonts`.
///
/// pygames `sysfont.py` fuehrt zwei Woerterbuecher der Gestalt
/// `{Name: {(fett, schraeg): Pfad}}`, und ein Programm darf sie lesen.
/// Hier steht die Tabelle im Kern, also gibt es diesen Zugang - gemessen
/// hat pygame nach dem ersten `match_font` **391** Schriften und **33**
/// Zweitnamen darin, und beide waren hier leer.
///
/// Die Namen kommen in der Reihenfolge von [`get_fonts`], die Schnitte je
/// Name in der Reihenfolge des Eintragens - wie ein Python-`dict`. Die
/// Zweitnamen sind sortiert: `sysfont.initsysfonts` baut sie auf der
/// Python-Seite ohnehin neu, wie pygame mit `create_aliases()`.
#[allow(clippy::type_complexity)]
pub fn tabellen() -> (
    Vec<(String, Vec<((bool, bool), PathBuf)>)>,
    Vec<(String, Vec<((bool, bool), PathBuf)>)>,
) {
    let t = tabelle();
    let fonts = t
        .reihenfolge
        .iter()
        .filter_map(|n| t.fonts.get(n).map(|s| (n.clone(), s.0.clone())))
        .collect();
    let mut alias: Vec<_> = t
        .alias
        .iter()
        .map(|(n, s)| (n.clone(), s.0.clone()))
        .collect();
    alias.sort();
    (fonts, alias)
}

/// Sucht die Datei zu einem Schriftnamen - `pygame.font.match_font`.
///
/// `name` darf eine Kommaliste sein (`"bahnschrift,segoeui,consolas"`);
/// genommen wird der erste Name, der sich aufloesen laesst.
pub fn match_font(name: &str, bold: bool, italic: bool) -> Option<PathBuf> {
    let t = tabelle();
    for einzeln in name.split(',') {
        let Some(s) = t.schnitte(einzeln) else {
            continue;
        };
        // Die Schleife aus sysfont.py: erst genau der Schnitt, dann ohne
        // kursiv, dann ohne fett, dann irgendeiner.
        let (mut b, mut i) = (bold, italic);
        loop {
            if let Some(p) = s.get(&(b, i)) {
                return Some(p.clone());
            }
            if i {
                i = false;
            } else if b {
                b = false;
            } else {
                // `list(styles.values())[0]` - der zuerst eingetragene.
                return s.erster().map(|(_, p)| p.clone());
            }
        }
    }
    None
}

/// Was `SysFont` ueber die Namensliste herausfindet:
/// (Datei, hat_schon_fett, hat_schon_kursiv).
pub fn schnitt_suchen(name: &str, bold: bool, italic: bool) -> (Option<PathBuf>, bool, bool) {
    let t = tabelle();
    for einzeln in name.split(',') {
        let Some(s) = t.schnitte(einzeln) else {
            continue;
        };
        let schlicht = s.get(&(false, false));
        let genau = s.get(&(bold, italic));
        return match (genau, schlicht) {
            (None, None) => {
                // Weder der gewuenschte noch der Grundschnitt ist da: pygame
                // nimmt irgendeinen und merkt sich, was davon schon passt.
                // `list(styles.items())[0]` - der zuerst eingetragene.
                let Some((stil, pfad)) = s.erster() else {
                    continue;
                };
                (Some(pfad.clone()), bold && stil.0, italic && stil.1)
            }
            (None, Some(p)) => (Some(p.clone()), false, false),
            (Some(p), _) => {
                if schlicht.map(|q| q != p).unwrap_or(true) {
                    (Some(p.clone()), bold, italic)
                } else {
                    (Some(p.clone()), false, false)
                }
            }
        };
    }
    (None, false, false)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn registry_namen_werden_wie_pygame_zerlegt() {
        assert_eq!(
            eintrag_zerlegen("Arial (TrueType)"),
            ("arial".to_string(), false, false)
        );
        assert_eq!(
            eintrag_zerlegen("Arial Narrow Bold Italic (TrueType)"),
            ("arial".to_string(), true, true)
        );
        assert_eq!(
            eintrag_zerlegen("Segoe UI Light (TrueType)"),
            ("segoeui".to_string(), false, false)
        );
        assert_eq!(
            eintrag_zerlegen("Courier New Bold (TrueType)"),
            ("couriernew".to_string(), true, false)
        );
        assert_eq!(
            eintrag_zerlegen(" Cambria Math (TrueType)"),
            ("cambriamath".to_string(), false, false)
        );
    }

    #[test]
    fn schlichter_name_wirft_sonderzeichen_weg() {
        assert_eq!(schlichter_name("Comic Sans MS"), "comicsansms");
        assert_eq!(schlichter_name("MS-Gothic_2"), "msgothic2");
    }

    /// `str.isalnum` kennt Umlaute - der Name behaelt sie. Das ist der
    /// Name, den `reg.exe` frueher zu `lucidasanstypewriterschrg` machte.
    #[test]
    fn schlichter_name_behaelt_umlaute() {
        assert_eq!(
            schlichter_name("Lucida Sans Typewriter Schr\u{e4}g"),
            "lucidasanstypewriterschr\u{e4}g"
        );
        assert_eq!(schlichter_name("\u{c4}RGER"), "\u{e4}rger");
    }

    /// Ueberschreiben laesst den Platz stehen, und "irgendeiner" ist der
    /// erste - wie `d[k] = v` und `list(d.items())[0]` in Python.
    #[test]
    fn schnitte_verhalten_sich_wie_ein_dict() {
        let mut s = Schnitte::default();
        s.setzen((true, false), PathBuf::from("b1"));
        s.setzen((false, false), PathBuf::from("r"));
        s.setzen((true, false), PathBuf::from("b2"));
        assert_eq!(
            s.0,
            vec![
                ((true, false), PathBuf::from("b2")),
                ((false, false), PathBuf::from("r")),
            ]
        );
        assert_eq!(s.erster().map(|e| e.0), Some((true, false)));
        assert_eq!(s.get(&(false, false)), Some(&PathBuf::from("r")));
        assert_eq!(s.get(&(true, true)), None);
    }
}
