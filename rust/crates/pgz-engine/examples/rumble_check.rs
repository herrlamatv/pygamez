//! Prueft den Vibrationspfad (`audio::rumble`), den der Pruefstand nie
//! betritt: XInput laden, Controller zaehlen, mit Haptik an vibrieren lassen
//! (ohne Controller passiert nichts), mit Haptik aus nichts tun.
//!
//!     cargo run -p pgz-engine --release --example rumble_check

use std::time::Instant;

use pgz_core::settings::Settings;
use pgz_engine::audio;

fn main() {
    let an = Settings { haptik: true, ..Settings::default() };
    let aus = Settings { haptik: false, ..Settings::default() };
    match audio::gamepads() {
        Some(n) => println!("XInput geladen, angeschlossene Controller: {n}"),
        None => println!("kein XInput auf diesem System"),
    }
    let t0 = Instant::now();
    audio::rumble(Some(&an), 120);
    println!("Haptik an:  rumble(120 ms) kehrte nach {:.2} ms zurueck", t0.elapsed().as_secs_f64() * 1000.0);
    let t0 = Instant::now();
    audio::rumble(Some(&aus), 120);
    println!("Haptik aus: rumble(120 ms) kehrte nach {:.2} ms zurueck", t0.elapsed().as_secs_f64() * 1000.0);
    std::thread::sleep(std::time::Duration::from_millis(200));
    audio::shutdown();
    println!("Motoren aus, sauber beendet");
}
