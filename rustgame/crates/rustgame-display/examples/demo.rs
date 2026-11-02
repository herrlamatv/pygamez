//! demo.rs
//! =======
//! Ein echtes Fenster, das zeigt, dass diese Kiste laeuft.
//!
//! ```text
//! cargo run -p rustgame-display --example demo
//! ```
//!
//! Zu sehen: ein paar Formen aus [`rustgame_core::draw`], ein Ball, der
//! umherspringt, ein Kreis, der dem Zeiger folgt, und ein Ring, der
//! aufleuchtet, solange eine Maustaste unten ist. Die zuletzt gedrueckte
//! Taste wandert in den Fenstertitel - so laesst sich mit blossem Auge
//! pruefen, ob die Tastenzuordnung stimmt.
//!
//! Escape schliesst das Fenster, genau wie das Kreuz.
//!
//! Wer es ohne Fenster laufen lassen will:
//!
//! ```text
//! RUSTGAME_VIDEODRIVER=dummy cargo run -p rustgame-display --example demo
//! ```
//!
//! Dann laeuft dieselbe Schleife kopflos durch und beendet sich nach
//! hundert Bildern - der Beweis, dass beide Hintergrundstuecke denselben
//! Weg gehen.

use rustgame_core::{draw, Rect};
use rustgame_display::{display, event, key, konstanten::*, mouse};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    display::init()?;
    let bild = display::set_mode((640, 480), RESIZABLE)?;
    display::set_caption("rustgame-display - Escape schliesst");

    let kopflos = display::get_driver()? == "dummy";
    // Ohne Fenster laeuft die Schleife hundert Bilder und hoert auf;
    // `RUSTGAME_DEMO_BILDER` setzt die Zahl und gilt auch mit Fenster -
    // so laesst sich der Lauf pruefen, ohne dass jemand Escape druecken muss.
    let grenze: Option<u32> = std::env::var("RUSTGAME_DEMO_BILDER")
        .ok()
        .and_then(|s| s.parse().ok())
        .or(if kopflos { Some(100) } else { None });
    if kopflos {
        println!("Kein Fenster (RUSTGAME_VIDEODRIVER=dummy).");
    } else {
        println!(
            "Fenster offen: {:?}, Treiber {}",
            display::get_window_size()?,
            display::get_driver()?
        );
    }

    let (breite, hoehe) = (640i32, 480i32);
    let mut ball = (80.0f32, 60.0f32);
    let mut tempo = (3.7f32, 2.9f32);
    let mut bilder = 0u32;
    let mut letzte_taste = String::new();

    'lauf: loop {
        // --- Ereignisse ------------------------------------------------
        for e in event::get() {
            match e.typ {
                QUIT => break 'lauf,
                KEYDOWN => {
                    if e.key() == Some(K_ESCAPE) {
                        break 'lauf;
                    }
                    if let Some(k) = e.key() {
                        letzte_taste = key::name(k).to_string();
                        display::set_caption(&format!(
                            "rustgame-display - zuletzt: {letzte_taste}"
                        ));
                    }
                }
                _ => {}
            }
        }

        // --- Rechnen ---------------------------------------------------
        ball.0 += tempo.0;
        ball.1 += tempo.1;
        if ball.0 < 30.0 || ball.0 > breite as f32 - 30.0 {
            tempo.0 = -tempo.0;
        }
        if ball.1 < 30.0 || ball.1 > hoehe as f32 - 30.0 {
            tempo.1 = -tempo.1;
        }
        let zeiger = mouse::get_pos();
        let (links, _, rechts) = mouse::get_pressed();
        // Die Pfeiltasten schieben das Rechteck - damit laesst sich die
        // Zuordnung minifb -> pygame mit blossem Auge nachpruefen.
        let tasten = key::get_pressed();
        let schub = (
            i32::from(tasten[K_RIGHT]) - i32::from(tasten[K_LEFT]),
            i32::from(tasten[K_DOWN]) - i32::from(tasten[K_UP]),
        );

        // --- Zeichnen --------------------------------------------------
        {
            let mut s = bild.write();
            s.fill((18, 20, 28));

            // Ein Raster, damit man sieht, dass die Flaeche wirklich neu ist.
            for x in (0..breite).step_by(40) {
                draw::line(&mut s, (30, 33, 44), (x, 0), (x, hoehe), 1);
            }
            for y in (0..hoehe).step_by(40) {
                draw::line(&mut s, (30, 33, 44), (0, y), (breite, y), 1);
            }

            // Ein gefuelltes und ein umrandetes Rechteck.
            draw::rect(
                &mut s,
                (70, 130, 220),
                Rect::new(60 + schub.0 * 60, 320 + schub.1 * 60, 120, 80),
                0,
            );
            draw::rect(&mut s, (240, 200, 90), Rect::new(220, 320, 120, 80), 4);

            // Ein Vieleck.
            draw::polygon(
                &mut s,
                (120, 220, 150),
                &[(430, 310), (500, 300), (560, 360), (520, 420), (440, 400)],
                0,
            );

            // Der springende Ball.
            draw::circle(
                &mut s,
                (240, 90, 90),
                (ball.0 as i32, ball.1 as i32),
                26,
                0,
            );

            // Der Zeiger, und ein Ring, solange eine Taste unten ist.
            draw::circle(&mut s, (255, 255, 255), zeiger, 8, 2);
            if links || rechts {
                let farbe = if links {
                    (120, 200, 255)
                } else {
                    (255, 160, 120)
                };
                draw::circle(&mut s, farbe, zeiger, 22, 3);
            }
        }

        display::flip()?;

        bilder += 1;
        if grenze.is_some_and(|g| bilder >= g) {
            break;
        }
        // Ohne Fenster gibt es nichts zu takten; mit Fenster reichen etwa
        // 60 Bilder in der Sekunde. `rustgame_core::time::Clock` gibt es
        // noch nicht, also tut es hier ein Schlaf.
        if !kopflos {
            std::thread::sleep(std::time::Duration::from_millis(16));
        }
    }

    println!("{bilder} Bilder gezeigt, zuletzt gedrueckt: {letzte_taste:?}");
    display::quit();
    Ok(())
}
