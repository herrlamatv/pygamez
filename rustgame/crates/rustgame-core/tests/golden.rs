//! golden.rs
//! =========
//! Vergleicht rustgame **pixelweise** mit echtem pygame-ce.
//!
//! Die Referenzbilder in `rustgame/tests/golden/` sind mit echtem pygame
//! erzeugt (`tools/gen_golden.py`, kopflos ueber `SDL_VIDEODRIVER=dummy`).
//! Dieser Test spielt dieselben Faelle mit rustgame nach und vergleicht.
//!
//! Weicht etwas ab, landet ein **Dreitafelbild** (erwartet | bekommen |
//! Unterschied achtfach verstaerkt) in `rustgame.output/golden-diff/`, und
//! die ersten abweichenden Stellen werden mit beiden Farbwerten genannt -
//! damit man sieht, *wo* es klemmt, statt zu raten.
//!
//! Faelle fuer Funktionen, die es noch nicht gibt, werden uebersprungen und
//! am Ende gezaehlt. So bleibt der Test gruen, waehrend die Abdeckung
//! waechst, und meldet trotzdem ehrlich, wie weit sie reicht.
//!
//! Das Gegenstueck ist `fuzz.rs`: dort wird nicht *eine* Funktion geprueft,
//! sondern ihr Zusammenspiel in gewuerfelten Folgen.

mod gemeinsam;

use std::collections::BTreeMap;

use gemeinsam::{
    abweichungen, anzahl_abweichungen, ausgabe_dir, gefuellt, golden_dir, lade_png, muster_auf,
    schreibe_dreitafel,
};
use rustgame_core::rect::Rect;
use rustgame_core::surface::{Surface, SRCALPHA};

/// Was ein Fall gezeichnet hat - oder warum nicht.
///
/// `Fehlt` ist zurzeit ungenutzt, und das ist eine **gute** Nachricht: es
/// heisst, dass der Generator fuer jeden der 62 Faelle eine fertige
/// Rust-Umsetzung gefunden hat. Die Variante bleibt trotzdem stehen -
/// sobald jemand einen Fall fuer etwas noch nicht Gebautes eintraegt,
/// wird sie wieder gebraucht, und ohne sie waere der Test rot statt
/// ehrlich uebersprungen.
#[allow(dead_code)]
enum Ergebnis {
    Fertig(Surface),
    /// Die noetige Funktion gibt es noch nicht.
    Fehlt(&'static str),
}

include!("gen/faelle.rs");

#[test]
fn rustgame_zeichnet_wie_pygame() {
    let dir = golden_dir();
    assert!(
        dir.join("manifest.json").exists(),
        "Referenzbilder fehlen. Erst erzeugen:\n  \
         .venv/Scripts/python.exe rustgame/tools/gen_golden.py"
    );

    let mut fehlend: BTreeMap<&str, usize> = BTreeMap::new();
    let mut geprueft = 0usize;
    let mut fehler: Vec<String> = Vec::new();

    for (name, tol, bauen) in faelle() {
        let pfad = dir.join(format!("{name}.png"));
        let Some(erwartet) = lade_png(&pfad) else {
            fehler.push(format!("{name}: Referenzbild nicht lesbar ({pfad:?})"));
            continue;
        };

        let bekommen = match bauen() {
            Ergebnis::Fehlt(grund) => {
                *fehlend.entry(grund).or_insert(0) += 1;
                continue;
            }
            Ergebnis::Fertig(s) => s,
        };
        geprueft += 1;

        let (bw, bh) = bekommen.get_size();
        if (bw, bh) != (erwartet.w, erwartet.h) {
            fehler.push(format!(
                "{name}: Groesse {bw}x{bh}, erwartet {}x{}",
                erwartet.w, erwartet.h
            ));
            continue;
        }

        let ist = bekommen.to_rgba_bytes();
        let anzahl = anzahl_abweichungen(&erwartet.daten, &ist, tol);
        if anzahl > 0 {
            schreibe_dreitafel(
                &ausgabe_dir("golden-diff").join(format!("{name}.png")),
                &erwartet.daten,
                &ist,
                bw,
                bh,
            );
            fehler.push(format!(
                "{name}: {anzahl} von {} Pixeln weichen ab (Toleranz {tol})\n      {}",
                bw * bh,
                abweichungen(&erwartet.daten, &ist, bw, tol, 10).join("\n      ")
            ));
        }
    }

    println!("\n{geprueft} Faelle pixelweise gegen echtes pygame geprueft.");
    if !fehlend.is_empty() {
        println!("Noch nicht umgesetzt (uebersprungen):");
        for (grund, n) in &fehlend {
            println!("   {n:2} x {grund}");
        }
    }

    assert!(
        fehler.is_empty(),
        "\n{} Fall/Faelle weichen von pygame ab:\n  {}\n\n\
         Differenzbilder liegen in rustgame.output/golden-diff/",
        fehler.len(),
        fehler.join("\n  ")
    );
}
