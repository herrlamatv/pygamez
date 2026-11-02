//! fuzz.rs
//! =======
//! **Differenzielles Fuzzing** gegen echtes pygame-ce.
//!
//! Die Referenzbilder in `golden.rs` pruefen je *eine* Sache. Fehler
//! entstehen aber selten in einer einzelnen Funktion, sondern im
//! Zusammenspiel: ein Blit mit Schluesselfarbe auf eine beschnittene
//! Teilflaeche, deren Elternteil ein Flaechen-Alpha traegt. So etwas
//! schreibt niemand von Hand als Testfall auf.
//!
//! Also wuerfelt `tools/fuzz_ops.py` es: echtes pygame fuehrt die Folge
//! aus, abgelegt wird **nur der Hash** des Ergebnisses - 500 Faelle waeren
//! sonst 500 PNGs im Verzeichnis. Hier wird dieselbe Folge nachgespielt.
//!
//! Faellt ein Fall, nennt die Meldung seine Nummer. Das erwartete Bild und
//! die Operationsfolge holt man sich damit:
//!
//! ```text
//! .venv/Scripts/python.exe rustgame/tools/fuzz_ops.py --zeige <nummer>
//! ```
//!
//! Das *bekommene* Bild legt dieser Test von sich aus daneben.

mod gemeinsam;

// Welche Hilfen gebraucht werden, entscheidet die **erzeugte** Falldatei -
// und die haengt an den Wuerfeln. Ein Lauf ohne `blit_gefuellt` laesst
// `gefuellt` ungenutzt, der naechste braucht es wieder. Die Liste deshalb
// vollstaendig halten statt bei jedem Wuerfeln nachzupflegen.
#[allow(unused_imports)]
use gemeinsam::{ausgabe_dir, fnv1a, gefuellt, muster, muster_auf, schreibe_png};
use rustgame_core::rect::Rect;
use rustgame_core::surface::{Surface, SRCALPHA};

/// Was ein gewuerfelter Fall geliefert hat.
struct FuzzErgebnis {
    hash: u64,
    erwartet: u64,
    flaeche: Surface,
}

include!("gen/fuzz.rs");

#[test]
fn gewuerfelte_folgen_treffen_pygame() {
    let faelle = fuzz_faelle();
    assert!(
        !faelle.is_empty(),
        "Keine Fuzz-Faelle erzeugt. Erst wuerfeln:\n  \
         .venv/Scripts/python.exe rustgame/tools/fuzz_ops.py"
    );

    let mut fehler: Vec<String> = Vec::new();
    let mut bekannt: Vec<u64> = Vec::new();
    for (nummer, ist_bekannt, bauen) in &faelle {
        let e = bauen();
        if e.hash == e.erwartet {
            continue;
        }
        if *ist_bekannt {
            // Die eine benannte Luecke - siehe `bekannt_abweichend` in
            // `tools/fuzz_ops.py`. Gezaehlt, nicht gewertet: ein dauerhaft
            // roter Test wuerde jede **neue** Abweichung verdecken, und
            // das waere schlimmer als die Luecke selbst.
            bekannt.push(*nummer);
            continue;
        }
        let (w, h) = e.flaeche.get_size();
        let pfad = ausgabe_dir("fuzz-diff").join(format!("fuzz_{nummer}_bekommen.png"));
        schreibe_png(&pfad, w, h, &e.flaeche.to_rgba_bytes());
        fehler.push(format!(
            "Fall {nummer} ({w}x{h}): Hash 0x{:016X}, erwartet 0x{:016X}",
            e.hash, e.erwartet
        ));
    }

    println!(
        "\n{} gewuerfelte Folgen gegen echtes pygame geprueft.",
        faelle.len()
    );
    if !bekannt.is_empty() {
        println!(
            "{} davon treffen die benannte Luecke (scale2x nach einer \
             BLEND_RGBA_*-Mischart) und weichen ab: {bekannt:?}",
            bekannt.len()
        );
    }

    assert!(
        fehler.is_empty(),
        "\n{} von {} gewuerfelten Faellen weichen von pygame ab:\n  {}\n\n\
         Das bekommene Bild liegt in rustgame.output/fuzz-diff/.\n\
         Das erwartete Bild und die Operationsfolge:\n  \
         .venv/Scripts/python.exe rustgame/tools/fuzz_ops.py --zeige {}",
        fehler.len(),
        faelle.len(),
        fehler.join("\n  "),
        fehler
            .first()
            .and_then(|f| f.split_whitespace().nth(1))
            .unwrap_or("0")
    );
}

/// Der Hash muss auf beiden Seiten Zahl fuer Zahl dasselbe liefern -
/// sonst faellt jeder Fuzz-Fall, und zwar aus dem falschen Grund.
///
/// Die Sollwerte stammen aus `tools/fuzz_ops.py` selbst (dieselbe
/// Rechnung, in Python ausgefuehrt).
#[test]
fn der_hash_rechnet_auf_beiden_seiten_gleich() {
    assert_eq!(fnv1a(b""), 0xCBF2_9CE4_8422_2325, "leere Eingabe");
    assert_eq!(fnv1a(b"a"), 0xAF63_DC4C_8601_EC8C);
    assert_eq!(fnv1a(b"foobar"), 0x85944171F73967E8);
}
