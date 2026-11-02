//! toene.rs
//! ========
//! Spielt ein paar der synthetisierten Klaenge nacheinander ab.
//!
//! Der Nachweis, dass **beide Wege gehen**:
//!
//! ```text
//! cargo run --example toene                            # es klingt
//! RUSTGAME_AUDIODRIVER=dummy cargo run --example toene # es laeuft lautlos durch
//! ```
//!
//! Der zweite Aufruf gibt dieselben Zahlen aus wie der erste - Laenge,
//! Laufzeit, Kanalbelegung. Nur zu hoeren ist nichts. Und auf einem
//! Rechner ganz ohne Soundkarte laeuft auch der erste durch, mit einer
//! Zeile Hinweis am Anfang.

use std::time::Duration;

use rustgame_mixer::{mixer, music, synth, Channel};

/// Welche Klaenge vorgefuehrt werden - quer durch die Wellenformen.
const FOLGE: &[&str] = &["click", "select", "point", "shoot", "explode", "win"];

fn main() {
    // Gemessen an pygame: `get_init()` ist vor `init()` None.
    println!("vor init:    get_init() = {:?}", mixer::get_init());

    match mixer::init(Default::default()) {
        Ok(()) => println!("init:        ok"),
        // Genau der Fall, um den sich die Kiste dreht: ein sauberer
        // Fehler, und alles Weitere geht trotzdem.
        Err(e) => println!("init:        {e} - es laeuft weiter, nur still"),
    }
    println!(
        "nach init:   get_init() = {:?}, Treiber = {:?}, Kanaele = {:?}",
        mixer::get_init(),
        mixer::get_driver().unwrap_or_default(),
        mixer::get_num_channels().unwrap_or(0)
    );
    println!();

    // -- Die Klaenge einzeln ------------------------------------------------

    for name in FOLGE {
        let Ok(klang) = synth::sound(name) else {
            continue;
        };
        klang.set_volume(0.6);
        let Some(kanal) = klang.play(0, 0, 0) else {
            println!("{name:10} kein freier Kanal");
            continue;
        };
        println!(
            "{name:10} Laenge {:.3} s  Kanal {}  Lautstaerke {:.4}",
            klang.get_length(),
            kanal.id(),
            klang.get_volume()
        );
        warten_bis_still(kanal);
    }

    // -- Zwei auf einmal, und eine Warteschlange ---------------------------

    println!();
    let click = synth::sound("click").expect("click gibt es immer");
    let lock = synth::sound("lock").expect("lock gibt es immer");
    if let (Ok(a), Ok(b)) = (Channel::new(0), Channel::new(1)) {
        let _ = a.play(&click, 2, 0, 0); // loops=2 heisst dreimal
        let _ = b.play(&lock, 0, 0, 0);
        let _ = b.queue(&click);
        println!(
            "gleichzeitig: mixer.get_busy() = {}, click auf {} Kanaelen",
            mixer::get_busy(),
            click.get_num_channels()
        );
        warten_bis_still(a);
        warten_bis_still(b);
    }

    // -- Ausblenden ---------------------------------------------------------

    println!();
    let gameover = synth::sound("gameover").expect("gameover gibt es immer");
    if let Some(kanal) = gameover.play(-1, 0, 0) {
        std::thread::sleep(Duration::from_millis(120));
        kanal.fadeout(200);
        let start = rustgame_core::time::get_ticks();
        warten_bis_still(kanal);
        println!("gameover:  endlos gestartet, nach 120 ms ausgeblendet ueber 200 ms");
        println!(
            "           still nach weiteren {} ms",
            rustgame_core::time::get_ticks() - start
        );
    }

    // -- Und dasselbe als Musik --------------------------------------------

    println!();
    if music::load_sound(&synth::sound("level").expect("level gibt es immer")).is_ok() {
        println!("music:     get_pos() vor play = {}", music::get_pos());
        let _ = music::play(0, 0.0, 0);
        std::thread::sleep(Duration::from_millis(100));
        println!(
            "           get_pos() nach 100 ms = {}, get_busy() = {}",
            music::get_pos(),
            music::get_busy()
        );
        while music::get_busy() {
            std::thread::sleep(Duration::from_millis(5));
        }
        println!("           get_pos() nach dem Ende = {}", music::get_pos());
    }

    mixer::quit();
    println!();
    println!("nach quit:   get_init() = {:?}", mixer::get_init());
}

/// Wartet, bis auf dem Kanal nichts mehr laeuft.
///
/// `get_busy()` fuehrt den Mixer dabei nach - ohne diese Abfrage wuerde
/// die Zeit zwar trotzdem vergehen, aber niemand wuerde es merken.
fn warten_bis_still(kanal: Channel) {
    while kanal.get_busy() {
        std::thread::sleep(Duration::from_millis(5));
    }
}
