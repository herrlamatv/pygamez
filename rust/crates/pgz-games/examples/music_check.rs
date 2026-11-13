//! Prueft den Musikpfad, den der Pruefstand nie betritt (dort ist Audio aus):
//! Geometry-Dash-Soundtrack bauen, ueber `audio::music_play` abspielen,
//! pausieren, Lautstaerke aendern, stoppen. Gespielt wird STUMM
//! (Ton aus in den Einstellungen).
//!
//!     cargo run -p pgz-games --release --example music_check

use std::time::Instant;

use pgz_core::settings::Settings;
use pgz_engine::audio;
use pgz_games::geodash_music as music;

fn main() {
    audio::init();
    let settings = Settings { sound: false, ..Settings::default() };
    for (i, style) in ["drive", "chip", "dream", "dark", "none"].iter().enumerate() {
        let t0 = Instant::now();
        let voices = music::build_voices(style, 0, &format!("lama-{i}"));
        let bau = t0.elapsed().as_secs_f64() * 1000.0;
        match voices {
            Some(v) => {
                let laenge: Vec<usize> = v.iter().map(|(s, _)| s.len()).collect();
                let ok = audio::music_play(&v, 4711, Some(&settings), 1.0);
                audio::music_set_volume(0.45);
                audio::music_tick(Some(4711), true, &settings);
                audio::music_tick(Some(4711), false, &settings);
                let laeuft = audio::music_playing(Some(4711));
                std::thread::sleep(std::time::Duration::from_millis(150));
                audio::music_stop(Some(4711));
                println!("{style:6} gebaut in {bau:7.1} ms, Stimmen {laenge:?}, gestartet {ok}, laeuft {laeuft}, gestoppt {}", !audio::music_playing(None));
            }
            None => println!("{style:6} keine Musik (wie Python)"),
        }
    }
    // Fremder Besitzer: music_tick mit anderem Spiel stoppt die Musik.
    let v = music::build_voices("drive", 128, "lama-launch").expect("drive");
    audio::music_play(&v, 1, Some(&settings), 1.0);
    audio::music_tick(Some(2), false, &settings);
    println!("Besitzerwechsel stoppt: {}", !audio::music_playing(None));
    audio::shutdown();
    println!("sauber beendet");
}
