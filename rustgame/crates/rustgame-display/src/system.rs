//! system.rs
//! =========
//! `pygame.system` - was der Rechner ueber sich selbst sagt.
//!
//! ## Zwei Aufrufe gehen, vier nicht - und warum das so bleibt
//!
//! `get_total_ram`, `get_power_state` und `get_pref_locales` brauchen
//! einen **Systemaufruf**: unter Windows `GlobalMemoryStatusEx`,
//! `GetSystemPowerStatus` und `GetUserPreferredUILanguages`. Die gibt es
//! nur ueber FFI, und FFI ist in Rust `unsafe`.
//!
//! Im ganzen Workspace steht **kein einziges `unsafe`**. Das ist keine
//! Ziererei: ein Nachbau, dessen Wert darin besteht, nachweisbar
//! dasselbe zu tun wie das Vorbild, sollte nicht ausgerechnet fuer drei
//! Auskuenfte am Rand die Sprachgarantien aufgeben, die ihn ueberhaupt
//! erst nachpruefbar machen.
//!
//! Also melden die drei einen **klaren Fehler mit Begruendung**. In der
//! Abdeckungstabelle heisst das "nur als klarer Fehler" und zaehlt
//! ausdruecklich als **fehlend** - die Zahl soll nicht schmeicheln. Ein
//! Programm, das sie ruft, erfaehrt den Grund, statt eine erfundene Zahl
//! zu bekommen und daran eine Weiche zu stellen.
//!
//! Was **ohne** Systemaufruf geht, geht auch wirklich:
//!
//! | Aufruf | wie |
//! |---|---|
//! | `get_cpu_instruction_sets()` | `is_x86_feature_detected!` aus der Standardbibliothek |
//! | `get_pref_path(org, app)` | aus `APPDATA` bzw. `XDG_DATA_HOME`, und der Ordner wird angelegt |

use std::path::PathBuf;

use crate::{Ergebnis, Fehler};

/// Die vierzehn Namen, die pygame meldet - in **seiner** Reihenfolge.
///
/// Gemessen an pygame-ce 2.5.7. Die Reihenfolge steht hier, weil ein
/// Woerterbuch in Python die Einfuegereihenfolge behaelt und ein
/// Vergleich sonst an der Sortierung scheitert.
const BEFEHLSSAETZE: [&str; 14] = [
    "ALTIVEC", "MMX", "SSE", "SSE2", "SSE3", "SSE41", "SSE42", "AVX", "AVX2", "AVX512F", "NEON",
    "ARMSIMD", "LSX", "LASX",
];

/// `pygame.system.get_cpu_instruction_sets()`.
///
/// Was die Standardbibliothek nicht kennt, ist `false` - und das ist
/// keine Vereinfachung: `ALTIVEC` (PowerPC), `LSX`/`LASX` (LoongArch)
/// und `ARMSIMD` gibt es auf x86 nicht, und `NEON` nur auf ARM.
pub fn get_cpu_instruction_sets() -> Vec<(&'static str, bool)> {
    BEFEHLSSAETZE
        .iter()
        .map(|name| (*name, kann(name)))
        .collect()
}

#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
fn kann(name: &str) -> bool {
    match name {
        "MMX" => is_x86_feature_detected!("mmx"),
        "SSE" => is_x86_feature_detected!("sse"),
        "SSE2" => is_x86_feature_detected!("sse2"),
        "SSE3" => is_x86_feature_detected!("sse3"),
        "SSE41" => is_x86_feature_detected!("sse4.1"),
        "SSE42" => is_x86_feature_detected!("sse4.2"),
        "AVX" => is_x86_feature_detected!("avx"),
        "AVX2" => is_x86_feature_detected!("avx2"),
        "AVX512F" => is_x86_feature_detected!("avx512f"),
        _ => false,
    }
}

#[cfg(target_arch = "aarch64")]
fn kann(name: &str) -> bool {
    // Auf aarch64 ist NEON Teil der Grundausstattung.
    name == "NEON"
}

#[cfg(not(any(target_arch = "x86", target_arch = "x86_64", target_arch = "aarch64")))]
fn kann(_name: &str) -> bool {
    false
}

/// `pygame.system.get_pref_path(org, app)`.
///
/// Gemessen an pygame unter Windows: `get_pref_path("MeinOrg",
/// "MeineApp")` gibt
/// `C:\Users\<du>\AppData\Roaming\MeinOrg\MeineApp\` - **mit**
/// Schlusstrenner -, und der Ordner **existiert danach**. Das Anlegen
/// gehoert also dazu und ist kein Beiwerk.
pub fn get_pref_path(org: &str, app: &str) -> Ergebnis<String> {
    let wurzel = wurzelordner()?;
    let mut pfad = PathBuf::from(wurzel);
    if !org.is_empty() {
        pfad.push(org);
    }
    pfad.push(app);
    std::fs::create_dir_all(&pfad).map_err(|_| {
        Fehler::Pygame("Failed to create the directory for the preference path")
    })?;
    let mut text = pfad.to_string_lossy().into_owned();
    if !text.ends_with(std::path::MAIN_SEPARATOR) {
        text.push(std::path::MAIN_SEPARATOR);
    }
    Ok(text)
}

/// Wo das System Anwendungsdaten hinlegt.
fn wurzelordner() -> Ergebnis<String> {
    if let Ok(v) = std::env::var("APPDATA") {
        if !v.is_empty() {
            return Ok(v);
        }
    }
    if let Ok(v) = std::env::var("XDG_DATA_HOME") {
        if !v.is_empty() {
            return Ok(v);
        }
    }
    if let Ok(v) = std::env::var("HOME") {
        if !v.is_empty() {
            return Ok(format!("{v}/.local/share"));
        }
    }
    Err(Fehler::Pygame("Failed to get the preference path"))
}

/// `pygame.system.get_total_ram()` - der Arbeitsspeicher in **Megabyte**.
///
/// ## Eine Grenze, die an einer Kiste hing
///
/// Hier stand ein Fehler mit der Begruendung, das brauche einen
/// Systemaufruf (`GlobalMemoryStatusEx` unter Windows, `sysinfo` unter
/// Linux), und ein Systemaufruf brauche FFI, und FFI sei in Rust `unsafe`.
/// Die ersten zwei Glieder stimmen, das dritte ist eine Verwechslung:
/// `unsafe` braucht **irgendwer**, aber nicht dieses Programm. Die Kiste
/// `sysinfo` tut es und bietet nach aussen eine sichere Schnittstelle an -
/// dasselbe Muster wie bei OPUS, WavPack und XM, wo dreimal "in reinem
/// Rust nicht in Reichweite" stand und dreimal falsch war.
///
/// Damit ist der letzte Eintrag der Liste "nur als klarer Fehler" fort.
///
/// ## Die Einheit ist Megabyte
///
/// Gemessen an pygames eigenem Test, der `assertGreater(ram, 0)` und
/// `assertIsInstance(ram, int)` prueft - und an SDLs `SDL_GetSystemRAM`,
/// dessen Dokumentation "the amount of RAM configured in the system in MiB"
/// sagt. Wer Bytes zurueckgibt, faellt dem Test nicht auf und einem
/// Programm, das `f"{ram} MB"` schreibt, sehr wohl.
pub fn get_total_ram() -> Ergebnis<u64> {
    let mut s = sysinfo::System::new();
    s.refresh_memory();
    // `total_memory()` gibt Bytes; SDL meldet MiB.
    Ok(s.total_memory() / (1024 * 1024))
}

/// `pygame.system.get_power_state()` - **`None`**, und das ist die Antwort.
///
/// pygame darf hier selbst nichts wissen, und sein eigener Test sagt es:
///
/// ```python
/// power_state = pygame.system.get_power_state()
/// self.assertIsInstance(power_state, (type(None), pygame.system.PowerState))
/// if power_state is not None:
///     ...
/// ```
///
/// `None` heisst "der Ladezustand ist nicht zu ermitteln" - genau das trifft
/// hier zu, denn `SDL_GetPowerInfo` braeuchte FFI und der Arbeitsbereich
/// hat kein `unsafe`. Hier stand ein Fehler, und der war die **falsche**
/// Auskunft: er sagt "das gibt es nicht", wo die Schnittstelle ein
/// "unbekannt" vorsieht. Ein Programm, das nach dem Akku fragt, soll
/// weiterlaufen und nicht abbrechen.
///
/// Die Klasse `PowerState` bleibt trotzdem da - siehe `rechner.rs`.
pub fn get_power_state() -> Ergebnis<Option<()>> {
    Ok(None)
}

/// `pygame.system.get_pref_locales()` - was die **Umgebung** sagt.
///
/// SDLs `SDL_GetPreferredLocales` braeuchte FFI; was ohne auskommt, sind
/// die ueblichen Umgebungsvariablen. Gelesen werden `LC_ALL`, `LC_MESSAGES`,
/// `LANG` und `LANGUAGE`, in dieser Reihenfolge - dieselbe, die die
/// C-Bibliothek selbst benutzt.
///
/// Auf Windows sind sie gewoehnlich **nicht gesetzt**, und dann kommt eine
/// leere Liste. Das ist keine Ausrede, sondern die Antwort: es ist nichts
/// eingestellt, was sich lesen liesse. pygames Test laeuft ueber die Liste
/// und prueft jeden Eintrag; eine leere Liste besteht ihn, weil es nichts
/// zu pruefen gibt.
///
/// Ein `"de_DE.UTF-8"` wird zu `("de", Some("DE"))`; die Kodierung hinter
/// dem Punkt faellt weg, wie bei pygame.
pub fn get_pref_locales() -> Ergebnis<Vec<(String, Option<String>)>> {
    let mut aus = Vec::new();
    for name in ["LC_ALL", "LC_MESSAGES", "LANG", "LANGUAGE"] {
        let Ok(wert) = std::env::var(name) else { continue };
        for teil in wert.split(':') {
            let ohne_kodierung = teil.split('.').next().unwrap_or(teil);
            if ohne_kodierung.is_empty() || ohne_kodierung == "C" || ohne_kodierung == "POSIX" {
                continue;
            }
            let mut stuecke = ohne_kodierung.split(['_', '-']);
            let sprache = stuecke.next().unwrap_or("").to_string();
            if sprache.len() < 2 {
                continue;
            }
            let land = stuecke.next().filter(|l| l.len() >= 2).map(|l| l.to_string());
            if !aus.iter().any(|(s, l): &(String, Option<String>)| {
                *s == sprache && *l == land
            }) {
                aus.push((sprache, land));
            }
        }
        if !aus.is_empty() {
            break;
        }
    }
    Ok(aus)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Die vierzehn Namen und ihre Reihenfolge sind an pygame gemessen.
    #[test]
    fn die_befehlssaetze_heissen_wie_bei_pygame() {
        let s = get_cpu_instruction_sets();
        assert_eq!(s.len(), 14);
        assert_eq!(s[0].0, "ALTIVEC");
        assert_eq!(s[13].0, "LASX");
        // Auf x86 ist SSE2 seit zwanzig Jahren Pflicht - wenn das hier
        // `false` waere, stimmte der Erkennungsweg nicht.
        #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
        assert!(s.iter().any(|(n, k)| *n == "SSE2" && *k));
        // Und was es auf dieser Bauart nicht gibt, meldet auch nichts.
        #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
        assert!(s.iter().any(|(n, k)| *n == "NEON" && !*k));
    }

    /// Der Ordner wird **angelegt** - gemessen an pygame.
    #[test]
    fn der_wunschpfad_entsteht_wirklich() {
        let p = get_pref_path("rustgame-test", "pfadprobe").unwrap();
        assert!(p.ends_with(std::path::MAIN_SEPARATOR), "Schlusstrenner: {p}");
        assert!(std::path::Path::new(&p).is_dir(), "angelegt: {p}");
        let _ = std::fs::remove_dir_all(&p);
    }

    /// Die drei Systemauskuenfte - **und jede sagt es anders**.
    ///
    /// Dieser Test ist dreimal umgeschrieben worden, und jedes Mal, weil
    /// eine Grenze fiel. Zuerst stand hier, alle drei melden einen Fehler;
    /// dann, `get_total_ram()` bleibe einer, weil es keine
    /// Ausweichantwort habe und ein Systemaufruf `unsafe` brauche.
    ///
    /// Auch das war falsch, und der Fehler war eine Verwechslung:
    /// `unsafe` braucht **irgendwer**, aber nicht dieses Programm. Die
    /// Kiste `sysinfo` tut es hinter einer sicheren Schnittstelle. Jetzt
    /// gibt es dort eine echte Zahl.
    ///
    /// Die Lehre steht hier, weil sie sich wiederholt hat: **ein Test, der
    /// eine Grenze festschreibt, verteidigt sie gegen die Messung.** Als
    /// `get_total_ram` echt wurde, schlug dieser Test an - und zwar nicht,
    /// weil etwas kaputt war, sondern weil er noch das alte Versprechen
    /// hielt.
    #[test]
    fn die_drei_systemauskuenfte_antworten() {
        let ram = get_total_ram().expect("sysinfo antwortet auf jeder Maschine");
        // In Megabyte, wie SDLs `SDL_GetSystemRAM`. Die Schranken sind
        // weit genug, um jede Maschine zu fassen, und eng genug, um eine
        // Verwechslung der Einheit zu fangen: waeren es Bytes, lage die
        // Zahl bei Milliarden.
        assert!(
            (64..4_194_304).contains(&ram),
            "Megabyte erwartet, bekam {ram}"
        );
        assert_eq!(get_power_state().unwrap(), None, "unbekannt heisst None");
        assert!(get_pref_locales().is_ok(), "liest die Umgebung");
    }

    /// `get_pref_locales` zerlegt einen Umgebungswert wie die C-Bibliothek.
    #[test]
    fn sprachen_aus_der_umgebung() {
        // Die Funktion liest echte Variablen; geprueft wird deshalb, dass
        // sie nie scheitert und nur brauchbare Eintraege liefert.
        for (sprache, land) in get_pref_locales().unwrap() {
            assert!(sprache.len() >= 2, "Sprachkuerzel zu kurz: {sprache}");
            assert!(!sprache.contains('.'), "Kodierung nicht abgetrennt");
            if let Some(l) = land {
                assert!(l.len() >= 2, "Landeskuerzel zu kurz: {l}");
            }
        }
    }
}
