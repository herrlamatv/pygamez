//! gegen_pygame.rs
//! ===============
//! Die Sollwerte in dieser Datei sind **an echtem pygame-ce 2.5.7
//! gemessen**, nicht erfunden.
//!
//! Gemessen wurde mit
//!
//! ```text
//! set SDL_VIDEODRIVER=dummy
//! set SDL_AUDIODRIVER=dummy
//! .venv\Scripts\python.exe <skript>
//! ```
//!
//! also ohne Soundkarte - genau wie diese Tests laufen.
//!
//! **Kein Test hier oeffnet ein Tongeraet.** [`vorbereiten`] schaltet den
//! stillen Treiber ausdruecklich ein, genau wie `vorbereiten` in
//! `rustgame-display` das fensterlose Hintergrundstueck einschaltet. Das
//! `cfg!(test)` in der Kiste selbst reicht dafuer **nicht**: ein
//! Integrationstest bindet die Bibliothek als gewoehnliche Abhaengigkeit
//! ein, und die ist ohne `cfg(test)` uebersetzt. Wer das vergisst,
//! bekommt einen Testlauf, der die Lautsprecher anwirft.
//!
//! Weil der Zustand dieser Kiste am Strang haengt und `cargo test` jeden
//! Test in einem eigenen Strang faehrt, stoeren sich die Tests nicht.
//!
//! Wo ein Sollwert ueberrascht, steht daneben, was gemessen wurde.

use std::time::Duration;

use rustgame_mixer::konstanten::*;
use rustgame_mixer::treiber::Wahl;
use rustgame_mixer::{ereignis, mixer, music, synth, Channel, Einstellungen, Fehler, Sound};

/// Stiller Treiber an - vor jedem `init()` in dieser Datei.
fn vorbereiten() {
    mixer::set_treiber(Wahl::Dummy);
}

/// Faehrt den Mixer still hoch und raeumt am Ende auf.
fn mit_mixer(f: impl FnOnce()) {
    vorbereiten();
    mixer::init(Einstellungen::default()).expect("der stille Treiber geht immer");
    f();
    mixer::quit();
}

/// Ein Klang bekannter Laenge, aus Abtastwerten gebaut.
fn klang(rahmen: usize) -> Sound {
    Sound::from_samples(&vec![1000i16; rahmen], 44_100, 1).unwrap()
}

/// Wartet, bis die Bedingung eintritt - hoechstens `grenze`.
fn warten(grenze: Duration, mut pruefen: impl FnMut() -> bool) -> Duration {
    let start = std::time::Instant::now();
    while !pruefen() && start.elapsed() < grenze {
        std::thread::sleep(Duration::from_millis(2));
    }
    start.elapsed()
}

// ---------------------------------------------------------------------------
//  init, get_init, quit
// ---------------------------------------------------------------------------

#[test]
fn vor_init_ist_get_init_none_und_get_busy_false() {
    // Gemessen: get_init() -> None. get_num_channels() wirft
    // "mixer not initialized", get_busy() aber NICHT - es gibt schlicht
    // False zurueck. Das ist die einzige Ausnahme.
    assert_eq!(mixer::get_init(), None);
    assert!(!mixer::get_busy());
    assert_eq!(
        mixer::get_num_channels().unwrap_err(),
        Fehler::NichtInitialisiert
    );
    assert_eq!(Sound::from_buffer(&[0; 4]).unwrap_err(), Fehler::NichtInitialisiert);
    assert_eq!(Channel::new(0).unwrap_err(), Fehler::NichtInitialisiert);
    // quit() ohne init() ist erlaubt und tut nichts.
    mixer::quit();
    mixer::quit();
}

#[test]
fn init_liefert_die_gemessenen_vorgaben() {
    mit_mixer(|| {
        // Gemessen: (44100, -16, 2) und 8 Kanaele.
        assert_eq!(mixer::get_init(), Some((44_100, -16, 2)));
        assert_eq!(mixer::get_num_channels().unwrap(), 8);
        assert!(!mixer::get_busy());
        // Der Puffer steht in get_init() nicht drin - gemessen sind es
        // drei Werte. Hier gibt es ihn zusaetzlich.
        assert_eq!(mixer::get_spec().unwrap().puffer, 512);
        // Ein zweites init() aendert nichts und beschwert sich nicht.
        assert!(mixer::init(Einstellungen::default()).is_ok());
        assert_eq!(mixer::get_init(), Some((44_100, -16, 2)));
    });
    assert_eq!(mixer::get_init(), None);
}

#[test]
fn init_mit_argumenten_wie_gemessen() {
    vorbereiten();
    // Alle Zeilen sind gemessene Paare aus (Argumente, get_init()).
    for (arg, soll) in [
        (Einstellungen::neu(22_050, 0, 0, 0), (22_050, -16, 2)),
        (Einstellungen::neu(0, -8, 0, 0), (44_100, -8, 2)),
        (Einstellungen::neu(0, 8, 0, 0), (44_100, 8, 2)),
        // Ueberraschung: size=32 kommt als -32 zurueck.
        (Einstellungen::neu(0, 32, 0, 0), (44_100, -32, 2)),
        (Einstellungen::neu(0, 0, 1, 0), (44_100, -16, 1)),
        (Einstellungen::neu(0, 0, 4, 0), (44_100, -16, 4)),
        (Einstellungen::neu(0, 0, 6, 0), (44_100, -16, 6)),
        (
            Einstellungen::neu(48_000, -16, 2, 2048),
            (48_000, -16, 2),
        ),
    ] {
        mixer::init(arg).unwrap();
        assert_eq!(mixer::get_init(), Some(soll), "fuer {arg:?}");
        // Gemessen: die Kanalzahl haengt nicht an den Argumenten.
        assert_eq!(mixer::get_num_channels().unwrap(), 8);
        mixer::quit();
    }
}

#[test]
fn pre_init_wirkt_erst_beim_naechsten_init() {
    vorbereiten();
    mixer::pre_init(Einstellungen::neu(22_050, -8, 1, 512));
    // Gemessen: get_init() bleibt None, pre_init faehrt nichts hoch.
    assert_eq!(mixer::get_init(), None);
    mixer::init(Einstellungen::default()).unwrap();
    assert_eq!(mixer::get_init(), Some((22_050, -8, 1)));
    mixer::quit();
    // Gemessen: pre_init(0,0,0,0) stellt die Vorgaben wieder her.
    mixer::pre_init(Einstellungen::default());
    mixer::init(Einstellungen::default()).unwrap();
    assert_eq!(mixer::get_init(), Some((44_100, -16, 2)));
    mixer::quit();
}

#[test]
fn im_testlauf_ist_der_treiber_still() {
    // Die Zusicherung, auf der die ganze Testdatei steht. Faellt sie,
    // laermt der naechste `cargo test` durch die Lautsprecher.
    mit_mixer(|| {
        assert_eq!(mixer::get_driver().unwrap(), "dummy");
    });
}

#[test]
fn sdl_mixer_fassung_meldet_den_umfang_den_sie_nachbaut() {
    // Hier stand `assert_eq!(get_sdl_mixer_version(), None)` mit der
    // Begruendung: diese Kiste bindet kein SDL_mixer ein, also waere
    // jede Zahl erfunden und wuerde Programme in die Irre fuehren, die
    // anhand von ihr eine Weiche stellen.
    //
    // Das klang richtig und war es nicht - **pygames eigene Testsuite
    // hat es widerlegt.** `image_test.py` schreibt die Regel fuer den
    // Schwesteraufruf aus:
    //
    //     # If get_extended() returns False then get_sdl_image_version()
    //     # should return None
    //
    // `None` heisst also nicht "die Bibliothek fehlt", sondern "der
    // Leistungsumfang fehlt". Der ist hier da. Und drei ganze
    // Testdateien (`image`, `font`, `mixer_music`) liessen sich mit
    // `None` nicht einmal laden - sie vergleichen die Fassung beim
    // Import und bekamen
    // `TypeError: '<' not supported between instances of 'NoneType'`.
    //
    // Gemeldet wird deshalb die Fassung, deren Umfang nachgebaut ist -
    // dieselbe Entscheidung wie bei `version.vernum`, das aus demselben
    // Grund `2.5.7` sagt. Was wirklich fehlt, sagt nicht diese Zahl,
    // sondern ein klarer Fehler an der Stelle, an der es fehlt.
    assert_eq!(SDL_MIXER_VERSION, (2, 8, 1), "gemessen an echtem pygame");
    assert_eq!(mixer::get_sdl_mixer_version(), Some(SDL_MIXER_VERSION));
}

// ---------------------------------------------------------------------------
//  Sound
// ---------------------------------------------------------------------------

#[test]
fn get_length_rechnet_in_einfacher_genauigkeit() {
    mit_mixer(|| {
        // Gemessen: 1000 Rahmen bei 44100 Hz geben 0.022675737738609314 -
        // und NICHT die mathematisch richtigen 0.022675736961451247.
        // pygame rechnet `(float)n / (float)freq` und weitet erst danach.
        assert_eq!(klang(1000).get_length(), 0.022_675_737_738_609_314);
        assert_ne!(klang(1000).get_length(), 1000.0 / 44_100.0);
        assert_eq!(klang(44_100).get_length(), 1.0);
        assert_eq!(klang(4410).get_length(), 0.100_000_001_490_116_12);
    });
}

#[test]
fn get_raw_liegt_im_mixerformat_vor() {
    mit_mixer(|| {
        // Gemessen: eine Mono-Datei mit 1000 Werten bei 44100 Hz liefert
        // 4000 Bytes - also Stereo mit 16 Bit, vier Bytes je Rahmen.
        assert_eq!(klang(1000).get_raw().len(), 4000);
        // Und bei 22050 Hz acht Bytes je Eingabewert, weil zusaetzlich
        // hochgerechnet wird.
        let halb = Sound::from_samples(&vec![1000i16; 11_025], 22_050, 1).unwrap();
        assert_eq!(halb.get_raw().len(), 88_200);
        assert_eq!(halb.get_length(), 0.5);
    });
}

#[test]
fn from_buffer_nimmt_rohe_werte_from_bytes_eine_datei() {
    mit_mixer(|| {
        let datei = synth::wav("click").unwrap();
        // Der ganze Unterschied: from_bytes liest den WAV-Kopf,
        // from_buffer haelt ihn fuer Klang. Gemessen kommt bei pygames
        // `Sound(buffer=wav_bytes)` genau das heraus - die Laenge stimmt
        // nicht, aber es gibt keinen Fehler.
        let a = Sound::from_bytes(&datei).unwrap();
        let b = Sound::from_buffer(&datei).unwrap();
        assert_eq!(a.get_raw().len(), 2205 * 4, "0.05 s Stereo bei 44100 Hz");
        // **Jedes Byte**, nicht nur ganze Werte. Hier stand
        // `datei.len() / 2 * 2` - die Kuerzung auf volle 16-Bit-Worte, die
        // die alte Ablage vornahm. Gemessen behaelt pygame alles: vier bis
        // neun Bytes hinein geben vier bis neun Bytes aus `get_raw()`, und
        // nur `get_length()` zaehlt ganze Rahmen.
        assert_eq!(b.get_raw().len(), datei.len());
        assert_ne!(a.get_length(), b.get_length());
    });
}

#[test]
fn set_volume_rastet_auf_128_stufen_ein() {
    mit_mixer(|| {
        let s = klang(100);
        // Alle Zahlen sind gemessene Paare.
        for (hinein, heraus) in [
            (0.0, 0.0),
            (0.5, 0.5),
            (1.0, 1.0),
            (1.5, 1.0),
            (0.333_333_3, 0.328_125),
            (0.1, 0.093_75),
            (0.01, 0.007_812_5),
            (1.0 / 255.0, 0.0),
        ] {
            s.set_volume(hinein);
            assert_eq!(s.get_volume(), heraus, "set_volume({hinein})");
        }
        // Ueberraschung: ein negativer Wert aendert gar nichts. Gemessen
        // gibt get_volume() danach 1.0 - den alten Wert.
        s.set_volume(1.0);
        s.set_volume(-0.5);
        assert_eq!(s.get_volume(), 1.0);
        s.set_volume(0.25);
        s.set_volume(-1.0);
        assert_eq!(s.get_volume(), 0.25);
    });
}

#[test]
fn sound_play_zaehlt_wiederholungen() {
    mit_mixer(|| {
        let s = klang(4410); // 0.1 s
                             // Gemessen mit demselben Klang:
                             // loops=0 -> 0.101 s, loops=1 -> 0.202 s,
                             // loops=2 -> 0.295 s, loops=3 -> 0.395 s.
        for (loops, soll_ms) in [(0, 100u64), (1, 200), (2, 300)] {
            let c = s.play(loops, 0, 0).unwrap();
            let gelaufen = warten(Duration::from_millis(soll_ms + 400), || !c.get_busy());
            let ist = gelaufen.as_millis() as u64;
            // Die untere Schranke laesst 5 ms Luft: `get_ticks()` zaehlt
            // in ganzen Millisekunden und schneidet ab, die Stoppuhr des
            // Tests laeuft erst nach `play()` los.
            assert!(
                ist + 5 >= soll_ms && ist < soll_ms + 120,
                "loops={loops}: {ist} ms statt rund {soll_ms} ms"
            );
        }
        // loops=-1 ist endlos.
        let c = s.play(-1, 0, 0).unwrap();
        std::thread::sleep(Duration::from_millis(350));
        assert!(c.get_busy(), "loops=-1 muss nach 350 ms noch laufen");
        c.stop();
    });
}

#[test]
fn maxtime_null_bedeutet_keine_schranke() {
    mit_mixer(|| {
        let s = klang(4410); // 0.1 s
                             // Gemessen: maxtime=0 laeuft die volle Laenge (0.976 s bei
                             // einem 1-Sekunden-Klang), maxtime=-1 ebenso.
        for maxtime in [0, -1] {
            let c = s.play(0, maxtime, 0).unwrap();
            let gelaufen = warten(Duration::from_millis(500), || !c.get_busy());
            assert!(
                gelaufen.as_millis() >= 95,
                "maxtime={maxtime} darf nicht frueh abbrechen ({} ms)",
                gelaufen.as_millis()
            );
        }
        // Gemessen: maxtime=50 bricht nach rund 55 ms ab.
        let c = s.play(0, 50, 0).unwrap();
        let gelaufen = warten(Duration::from_millis(400), || !c.get_busy());
        assert!(
            (40..100).contains(&(gelaufen.as_millis() as u64)),
            "maxtime=50: {} ms",
            gelaufen.as_millis()
        );
    });
}

#[test]
fn play_gibt_none_wenn_alle_kanaele_belegt_sind() {
    mit_mixer(|| {
        mixer::set_num_channels(2).unwrap();
        let s = klang(44_100); // 1 s
        assert_eq!(s.play(0, 0, 0).map(|c| c.id()), Some(0));
        assert_eq!(s.play(0, 0, 0).map(|c| c.id()), Some(1));
        // Gemessen: kein Fehler, sondern None - und get_num_channels()
        // bleibt bei 2 stehen.
        assert_eq!(s.play(0, 0, 0), None);
        assert_eq!(s.get_num_channels(), 2);
        assert!(mixer::get_busy());
        // find_channel() liefert dann ebenfalls None, mit force den
        // aeltesten.
        assert_eq!(mixer::find_channel(false).unwrap(), None);
        assert!(mixer::find_channel(true).unwrap().is_some());
        mixer::stop().unwrap();
        assert!(!mixer::get_busy());
        assert_eq!(s.get_num_channels(), 0);
    });
}

#[test]
fn find_channel_mit_force_und_ohne_kanaele_wirft() {
    mit_mixer(|| {
        mixer::set_num_channels(0).unwrap();
        assert_eq!(mixer::get_num_channels().unwrap(), 0);
        // Gemessen an echtem pygame nach `set_num_channels(0)`:
        //   find_channel()     -> None
        //   find_channel(True) -> IndexError: invalid channel index
        //
        // Der Grund steht in `mixer_find_channel` (src_c/mixer.c): es
        // reicht die -1 von `Mix_GroupOldest` ungeprueft an
        // `pgChannel_New` weiter, und das lehnt sie ab. Diese Kiste gab
        // hier zunaechst `None` zurueck - aufgefallen erst im Vergleich
        // ueber die Python-Huelle.
        assert_eq!(mixer::find_channel(false).unwrap(), None);
        assert_eq!(
            mixer::find_channel(true),
            Err(Fehler::UngueltigerKanal(usize::MAX))
        );
    });
}

// ---------------------------------------------------------------------------
//  Channel
// ---------------------------------------------------------------------------

#[test]
fn channel_grundzustand_wie_gemessen() {
    mit_mixer(|| {
        let c = Channel::new(0).unwrap();
        assert!(!c.get_busy());
        assert_eq!(c.get_sound(), None);
        assert_eq!(c.get_queue(), None);
        assert_eq!(c.get_volume(), 1.0);
        // Gemessen: get_endevent() ist 0 (NOEVENT), nicht etwa None.
        assert_eq!(c.get_endevent(), NOEVENT);
        // Gemessen: Channel(8) bei acht Kanaelen wirft
        // "IndexError: invalid channel index".
        assert_eq!(Channel::new(8).unwrap_err(), Fehler::UngueltigerKanal(8));
        assert_eq!(
            Channel::new(8).unwrap_err().to_string(),
            "invalid channel index"
        );
    });
}

#[test]
fn zwei_werte_bei_set_volume_sind_die_balance() {
    mit_mixer(|| {
        let c = Channel::new(0).unwrap();
        // Das ist die groesste Ueberraschung der ganzen Messreihe:
        // set_volume(l, r) stellt Mix_SetPanning ein und setzt die
        // Lautstaerke dabei auf voll. get_volume() gibt danach IMMER 1.0,
        // egal welche Werte man uebergeben hat, und die Balance laesst
        // sich nirgends mehr auslesen.
        for (links, rechts) in [(0.3, 0.7), (0.0, 1.0), (0.25, 0.25), (0.75, 0.25)] {
            c.set_volume_stereo(links, rechts);
            assert_eq!(
                c.get_volume(),
                1.0,
                "set_volume({links}, {rechts}) muss 1.0 ergeben"
            );
        }
        // Mit einem Wert verhaelt es sich normal.
        c.set_volume(0.5);
        assert_eq!(c.get_volume(), 0.5);
        c.set_volume(0.333_333_3);
        assert_eq!(c.get_volume(), 0.328_125);
    });
}

#[test]
fn pausierter_kanal_gilt_weiter_als_belegt() {
    mit_mixer(|| {
        let s = klang(44_100);
        let c = Channel::new(0).unwrap();
        c.play(&s, 0, 0, 0).unwrap();
        c.pause();
        // Gemessen: True. SDL_mixers Mix_Playing zaehlt Pausen mit.
        assert!(c.get_busy());
        assert!(mixer::get_busy());
        // Und die Zeit steht waehrend der Pause still.
        std::thread::sleep(Duration::from_millis(80));
        c.unpause();
        assert!(c.get_busy());
        c.stop();
        assert!(!c.get_busy());
        assert_eq!(c.get_sound(), None);
    });
}

#[test]
fn pause_haelt_die_uhr_an() {
    mit_mixer(|| {
        let s = klang(4410); // 0.1 s
        let c = Channel::new(0).unwrap();
        c.play(&s, 0, 0, 0).unwrap();
        std::thread::sleep(Duration::from_millis(30));
        c.pause();
        std::thread::sleep(Duration::from_millis(200));
        assert!(c.get_busy(), "waehrend der Pause darf nichts enden");
        c.unpause();
        let rest = warten(Duration::from_millis(400), || !c.get_busy());
        // Es fehlten rund 70 ms - die Pause zaehlt nicht mit.
        assert!(
            (50..130).contains(&(rest.as_millis() as u64)),
            "nach der Pause blieben {} ms statt rund 70",
            rest.as_millis()
        );
    });
}

#[test]
fn queue_auf_leerem_kanal_spielt_sofort() {
    mit_mixer(|| {
        let a = klang(4410);
        let c = Channel::new(0).unwrap();
        c.queue(&a).unwrap();
        // Gemessen: get_sound() ist der Klang, get_queue() ist None.
        assert_eq!(c.get_sound(), Some(a.clone()));
        assert_eq!(c.get_queue(), None);
        assert!(c.get_busy());
        c.stop();
    });
}

#[test]
fn queue_rueckt_nach_dem_ende_auf() {
    mit_mixer(|| {
        let a = klang(4410); // 0.1 s
        let b = klang(4410);
        let c = Channel::new(0).unwrap();
        c.play(&a, 0, 0, 0).unwrap();
        c.queue(&b).unwrap();
        assert_eq!(c.get_queue(), Some(b.clone()));
        // Gemessen: play(0.1 s) + queue(0.1 s) laeuft 0.197 s.
        let gesamt = warten(Duration::from_millis(600), || !c.get_busy());
        assert!(
            (180..300).contains(&(gesamt.as_millis() as u64)),
            "zusammen {} ms statt rund 200",
            gesamt.as_millis()
        );
    });
}

#[test]
fn stop_laesst_die_warteschlange_aufruecken() {
    mit_mixer(|| {
        // Die zweite grosse Ueberraschung: Channel.stop() ist kein
        // blosser Abbruch. Gemessen ist danach get_busy() == True und
        // get_sound() der Klang aus der Schlange - SDL ruft den
        // Fertig-Rueckruf auch beim Abbrechen, und pygames Rueckruf laesst
        // die Schlange aufruecken.
        let lang = klang(44_100);
        let kurz = klang(4410);
        let c = Channel::new(0).unwrap();
        c.play(&lang, 0, 0, 0).unwrap();
        c.queue(&kurz).unwrap();
        c.stop();
        assert!(c.get_busy(), "gemessen: nach stop() laeuft die Schlange");
        assert_eq!(c.get_sound(), Some(kurz));
        assert_eq!(c.get_queue(), None);
        // Und ohne Schlange ist stop() ein gewoehnlicher Abbruch.
        c.stop();
        assert!(!c.get_busy());
        assert_eq!(c.get_sound(), None);
    });
}

#[test]
fn channel_play_laesst_die_lautstaerke_stehen_sound_play_nicht() {
    mit_mixer(|| {
        let s = klang(44_100);
        let c = Channel::new(0).unwrap();
        c.set_volume(0.25);
        c.play(&s, 0, 0, 0).unwrap();
        // Gemessen: Channel.play ruehrt die Lautstaerke nicht an.
        assert_eq!(c.get_volume(), 0.25);
        c.stop();

        c.set_volume(0.25);
        let benutzt = s.play(0, 0, 0).unwrap();
        // Gemessen: Sound.play setzt Mix_Volume(ch, 128) - der Kanal
        // steht danach wieder auf voll.
        assert_eq!(benutzt.id(), 0);
        assert_eq!(benutzt.get_volume(), 1.0);
        mixer::stop().unwrap();
    });
}

// ---------------------------------------------------------------------------
//  Kanalwahl und Reservierung
// ---------------------------------------------------------------------------

#[test]
fn kanaele_werden_der_reihe_nach_vergeben() {
    mit_mixer(|| {
        mixer::set_num_channels(4).unwrap();
        let s = klang(44_100);
        // Gemessen: 0, 1, 2, 3, dann None.
        for erwartet in 0..4 {
            assert_eq!(s.play(0, 0, 0).map(|c| c.id()), Some(erwartet));
        }
        assert_eq!(s.play(0, 0, 0), None);
        // Wird einer frei, kommt genau der wieder dran.
        Channel::new(2).unwrap().stop();
        assert_eq!(s.play(0, 0, 0).map(|c| c.id()), Some(2));
        mixer::stop().unwrap();
    });
}

#[test]
fn reservierte_kanaele_gehen_nur_find_channel_nichts_an() {
    mit_mixer(|| {
        mixer::set_num_channels(6).unwrap();
        assert_eq!(mixer::set_reserved(2).unwrap(), 2);
        let s = klang(44_100);
        // Gemessen: Sound.play() faengt bei 2 an.
        assert_eq!(s.play(0, 0, 0).map(|c| c.id()), Some(2));
        assert_eq!(s.play(0, 0, 0).map(|c| c.id()), Some(3));
        mixer::stop().unwrap();
        // Gemessen und ueberraschend: find_channel() gibt trotzdem 0
        // zurueck - es ueberspringt reservierte Kanaele NICHT.
        assert_eq!(mixer::find_channel(false).unwrap().map(|c| c.id()), Some(0));
        // Ueber Channel(0) laesst sich ein reservierter Kanal weiter
        // bespielen - genau dafuer ist er reserviert.
        let c = Channel::new(0).unwrap();
        c.play(&s, 0, 0, 0).unwrap();
        assert!(c.get_busy());
        // Gemessen: mehr reservieren als es Kanaele gibt, gibt die
        // Kanalzahl zurueck.
        assert_eq!(mixer::set_reserved(99).unwrap(), 6);
        mixer::stop().unwrap();
    });
}

#[test]
fn set_num_channels_verkleinert_und_verstummt() {
    mit_mixer(|| {
        let s = klang(44_100);
        for _ in 0..3 {
            s.play(0, 0, 0);
        }
        assert_eq!(s.get_num_channels(), 3);
        mixer::set_num_channels(2).unwrap();
        // Gemessen: die Kanalzahl ist 2 und nur noch zwei Toene laufen.
        assert_eq!(mixer::get_num_channels().unwrap(), 2);
        assert_eq!(s.get_num_channels(), 2);
        assert_eq!(Channel::new(2).unwrap_err(), Fehler::UngueltigerKanal(2));
        // Gemessen: 0 ist erlaubt.
        mixer::set_num_channels(0).unwrap();
        assert_eq!(mixer::get_num_channels().unwrap(), 0);
        assert!(!mixer::get_busy());
    });
}

#[test]
fn endevent_ueberlebt_ein_verkleinern() {
    mit_mixer(|| {
        Channel::new(5).unwrap().set_endevent(USEREVENT + 2);
        mixer::set_num_channels(2).unwrap();
        mixer::set_num_channels(8).unwrap();
        // pygames channeldata waechst nur, es schrumpft nie - der Wert
        // steht also noch da.
        assert_eq!(Channel::new(5).unwrap().get_endevent(), USEREVENT + 2);
    });
}

// ---------------------------------------------------------------------------
//  Ausblenden
// ---------------------------------------------------------------------------

#[test]
fn fadeout_endet_nach_der_angegebenen_zeit() {
    mit_mixer(|| {
        let s = klang(44_100); // 1 s
        let c = s.play(-1, 0, 0).unwrap();
        std::thread::sleep(Duration::from_millis(30));
        c.fadeout(120);
        // Gemessen: Channel.fadeout(120) endete nach 0.126 s.
        let bis_still = warten(Duration::from_millis(800), || !c.get_busy());
        assert!(
            (100..260).contains(&(bis_still.as_millis() as u64)),
            "fadeout(120): {} ms",
            bis_still.as_millis()
        );

        // Und dasselbe ueber den Mixer.
        let c = s.play(-1, 0, 0).unwrap();
        mixer::fadeout(150).unwrap();
        let bis_still = warten(Duration::from_millis(800), || !mixer::get_busy());
        assert!(
            (130..300).contains(&(bis_still.as_millis() as u64)),
            "mixer.fadeout(150): {} ms",
            bis_still.as_millis()
        );
        assert!(!c.get_busy());
    });
}

// ---------------------------------------------------------------------------
//  Endereignisse
// ---------------------------------------------------------------------------

#[test]
fn endereignis_traegt_den_kanalindex() {
    mit_mixer(|| {
        let s = klang(4410);
        let c = Channel::new(3).unwrap();
        c.set_endevent(USEREVENT);
        c.play(&s, 0, 0, 0).unwrap();
        warten(Duration::from_millis(500), || !c.get_busy());
        let ereignisse = ereignis::abholen();
        assert_eq!(ereignisse.len(), 1);
        // Gemessen: das Ereignis traegt ein Feld `code` mit dem
        // Kanalindex - im Test war es {'code': 4} fuer Channel(4).
        assert_eq!(ereignisse[0].typ, USEREVENT);
        assert_eq!(ereignisse[0].code, Some(3));
        // Nach dem Abholen ist die Sammlung leer.
        assert!(ereignis::abholen().is_empty());
    });
}

#[test]
fn endereignis_faellt_auch_bei_stop_und_fadeout_an() {
    mit_mixer(|| {
        let s = klang(44_100);
        let c = Channel::new(0).unwrap();
        c.set_endevent(USEREVENT);
        // Gemessen: nach Channel.stop() liegt USEREVENT in der Schlange.
        c.play(&s, 0, 0, 0).unwrap();
        c.stop();
        assert_eq!(ereignis::abholen().len(), 1);
        // Gemessen: nach mixer.stop() genauso.
        c.play(&s, 0, 0, 0).unwrap();
        mixer::stop().unwrap();
        assert_eq!(ereignis::abholen().len(), 1);
        // Und nach einer Ausblendung.
        c.play(&s, 0, 0, 0).unwrap();
        c.fadeout(30);
        warten(Duration::from_millis(400), || !c.get_busy());
        assert_eq!(ereignis::abholen().len(), 1);
        // Ohne gesetztes Ereignis kommt nichts.
        c.set_endevent(NOEVENT);
        c.play(&s, 0, 0, 0).unwrap();
        c.stop();
        assert!(ereignis::abholen().is_empty());
    });
}

#[test]
fn die_senke_bekommt_die_ereignisse_sofort() {
    use std::cell::RefCell;
    use std::rc::Rc;

    mit_mixer(|| {
        // Genau so soll rustgame-display sich anmelden.
        let gesehen = Rc::new(RefCell::new(Vec::new()));
        let mitschrift = gesehen.clone();
        ereignis::set_senke(move |e| mitschrift.borrow_mut().push(e));

        let s = klang(4410);
        let c = Channel::new(1).unwrap();
        c.set_endevent(USEREVENT + 5);
        c.play(&s, 0, 0, 0).unwrap();
        warten(Duration::from_millis(500), || !c.get_busy());

        assert_eq!(gesehen.borrow().len(), 1);
        assert_eq!(gesehen.borrow()[0].typ, USEREVENT + 5);
        assert_eq!(gesehen.borrow()[0].code, Some(1));
        // Solange eine Senke haengt, sammelt der Mixer nichts mehr.
        assert!(ereignis::abholen().is_empty());
        ereignis::senke_loeschen();
    });
}

// ---------------------------------------------------------------------------
//  Musik
// ---------------------------------------------------------------------------

#[test]
fn musik_lautstaerke_steht_nach_init_auf_127_von_128() {
    mit_mixer(|| {
        // Gemessen: 0.9921875, nicht 1.0. pygames _init ruft zum Schluss
        // Mix_VolumeMusic(127).
        assert_eq!(music::get_volume(), 0.992_187_5);
        music::set_volume(0.5).unwrap();
        assert_eq!(music::get_volume(), 0.5);
        music::set_volume(0.333_333_3).unwrap();
        assert_eq!(music::get_volume(), 0.328_125);
        // Negativ aendert nichts - wie ueberall.
        music::set_volume(-1.0).unwrap();
        assert_eq!(music::get_volume(), 0.328_125);
    });
}

#[test]
fn musik_ohne_load_beschwert_sich_wie_gemessen() {
    mit_mixer(|| {
        // Gemessen: play() -> "music not loaded", set_pos() ->
        // "Music isn't playing", get_metadata() -> "music not loaded".
        // rewind, stop, pause, unpause, fadeout und unload gehen
        // stillschweigend durch.
        assert_eq!(music::play(0, 0.0, 0).unwrap_err(), Fehler::NichtGeladen);
        assert_eq!(music::set_pos(0.0).unwrap_err(), Fehler::SpieltNicht);
        assert_eq!(music::get_metadata().unwrap_err(), Fehler::NichtGeladen);
        assert!(music::rewind().is_ok());
        assert!(music::stop().is_ok());
        assert!(music::pause().is_ok());
        assert!(music::unpause().is_ok());
        assert!(music::fadeout(10).is_ok());
        assert!(music::unload().is_ok());
        assert_eq!(
            music::play(0, 0.0, 0).unwrap_err().to_string(),
            "music not loaded"
        );
    });
}

#[test]
fn musik_loops_zaehlt_anders_als_sound_loops() {
    mit_mixer(|| {
        let stueck = klang(4410); // 0.1 s
        music::load_sound(&stueck).unwrap();
        // Gemessen: loops=0 -> 1x, loops=1 -> 1x (!), loops=2 -> 2x.
        // Bei Sound.play bedeutet loops=1 dagegen 2x.
        for (loops, soll_ms) in [(0, 100u64), (1, 100), (2, 200), (3, 300)] {
            music::play(loops, 0.0, 0).unwrap();
            let gelaufen = warten(Duration::from_millis(soll_ms + 400), || !music::get_busy());
            let ist = gelaufen.as_millis() as u64;
            assert!(
                ist + 5 >= soll_ms && ist < soll_ms + 120,
                "music loops={loops}: {ist} ms statt rund {soll_ms} ms"
            );
        }
    });
}

#[test]
fn musik_get_pos_ist_eine_stoppuhr() {
    mit_mixer(|| {
        let stueck = klang(44_100); // 1 s
                                    // Gemessen: -1, solange nie gespielt wurde - auch nach load().
        assert_eq!(music::get_pos(), -1);
        music::load_sound(&stueck).unwrap();
        assert_eq!(music::get_pos(), -1);

        music::play(0, 0.0, 0).unwrap();
        // Gemessen: direkt nach play() ist es 0.
        assert!(music::get_pos() < 20);
        std::thread::sleep(Duration::from_millis(150));
        let nach_150 = music::get_pos();
        assert!(
            (140..220).contains(&nach_150),
            "nach 150 ms: {nach_150}"
        );

        // Gemessen: waehrend einer Pause bleibt der Zaehler stehen - und
        // get_busy() ist dabei False, anders als beim Kanal.
        music::pause().unwrap();
        assert!(!music::get_busy());
        let bei_pause = music::get_pos();
        std::thread::sleep(Duration::from_millis(120));
        assert_eq!(music::get_pos(), bei_pause);
        music::unpause().unwrap();
        assert!(music::get_busy());

        // Gemessen: rewind() setzt den Zaehler NICHT zurueck.
        let vor_rewind = music::get_pos();
        music::rewind().unwrap();
        assert!(
            music::get_pos() >= vor_rewind,
            "rewind darf get_pos nicht zuruecksetzen"
        );
        music::stop().unwrap();
    });
}

#[test]
fn musik_stop_wirft_die_warteschlange_weg() {
    mit_mixer(|| {
        let lang = klang(44_100);
        let kurz = klang(4410);
        music::load_sound(&lang).unwrap();
        music::queue_sound(&kurz, 0).unwrap();
        music::play(0, 0.0, 0).unwrap();
        std::thread::sleep(Duration::from_millis(30));
        music::stop().unwrap();
        // Gemessen: False. Anders als beim Kanal rueckt hier nichts auf -
        // pygames music_stop befreit die Schlange ausdruecklich vorher.
        assert!(!music::get_busy());
        assert_eq!(music::get_pos(), -1);
    });
}

#[test]
fn musik_warteschlange_rueckt_am_ende_auf() {
    mit_mixer(|| {
        let a = klang(4410); // 0.1 s
        let b = klang(4410);
        music::load_sound(&a).unwrap();
        music::queue_sound(&b, 0).unwrap();
        music::play(0, 0.0, 0).unwrap();
        // Gemessen: play + queue lief 0.206 s.
        let gesamt = warten(Duration::from_millis(700), || !music::get_busy());
        assert!(
            (180..320).contains(&(gesamt.as_millis() as u64)),
            "zusammen {} ms statt rund 200",
            gesamt.as_millis()
        );
    });
}

#[test]
fn musik_endereignis_traegt_keinen_code() {
    mit_mixer(|| {
        let stueck = klang(4410);
        music::load_sound(&stueck).unwrap();
        music::set_endevent(USEREVENT + 1);
        assert_eq!(music::get_endevent(), USEREVENT + 1);
        music::play(0, 0.0, 0).unwrap();
        warten(Duration::from_millis(500), || !music::get_busy());
        let e = ereignis::abholen();
        assert_eq!(e.len(), 1);
        assert_eq!(e[0].typ, USEREVENT + 1);
        // Gemessen: das Musikereignis kommt ohne Felder - {}, waehrend
        // das Kanalereignis {'code': n} traegt.
        assert_eq!(e[0].code, None);
        music::set_endevent(NOEVENT);
        assert_eq!(music::get_endevent(), NOEVENT);
    });
}

#[test]
fn musik_metadaten_haben_immer_vier_felder() {
    mit_mixer(|| {
        music::load_sound(&klang(100)).unwrap();
        // Gemessen fuer eine WAV-Datei: alle vier Felder leer.
        let m = music::get_metadata().unwrap();
        assert!(m.title.is_empty());
        assert!(m.album.is_empty());
        assert!(m.artist.is_empty());
        assert!(m.copyright.is_empty());
    });
}

#[test]
fn unload_nimmt_das_stueck_wieder_weg() {
    mit_mixer(|| {
        music::load_sound(&klang(4410)).unwrap();
        assert!(music::play(0, 0.0, 0).is_ok());
        music::unload().unwrap();
        assert!(!music::get_busy());
        // Gemessen: danach wieder "music not loaded".
        assert_eq!(music::play(0, 0.0, 0).unwrap_err(), Fehler::NichtGeladen);
    });
}

// ---------------------------------------------------------------------------
//  Formate
// ---------------------------------------------------------------------------

#[test]
fn wav_geht_aus_datei_und_aus_dem_puffer() {
    mit_mixer(|| {
        let daten = synth::wav("point").unwrap();
        let aus_puffer = Sound::from_bytes(&daten).unwrap();

        let mut pfad = std::env::temp_dir();
        pfad.push(format!("rustgame-mixer-test-{}.wav", std::process::id()));
        std::fs::write(&pfad, &daten).unwrap();
        let aus_datei = Sound::new(&pfad).unwrap();
        let _ = std::fs::remove_file(&pfad);

        assert_eq!(aus_datei.get_raw(), aus_puffer.get_raw());
        assert_eq!(aus_datei.get_length(), aus_puffer.get_length());
        // 0.18 s bei 44100 Hz sind 7938 Rahmen.
        assert_eq!(aus_datei.rahmen(), 7938);
    });
}

/// Die Beispieldaten des echten pygame - wenn die venv des Projekts da
/// ist.
///
/// Sie liegt bei `<pygamez>/.venv`, also drei Ebenen ueber dieser Kiste.
/// Fehlt sie, ueberspringen die Tests darauf still: ein Testlauf darf
/// nicht daran haengen, dass jemand anderes pygame installiert hat.
fn pygame_beispiel(name: &str) -> Option<std::path::PathBuf> {
    let mut p = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    for _ in 0..3 {
        p.pop();
    }
    p.push(".venv/Lib/site-packages/pygame/examples/data");
    p.push(name);
    if p.is_file() { Some(p) } else { None }
}

/// **Jetzt liest sie alles, was pygames Testsuite ihr hinlegt.**
///
/// Diese Stelle ist die Chronik einer Liste, die dreimal kuerzer wurde:
///
/// ```text
/// zuerst   mp3, flac       - bis rodios Merkmale angeschaltet wurden
/// dann     ADPCM-WAV       - bis `wav.rs` es selbst auspackte
/// dann     WavPack, XM     - bis `symphonia-codec-wavpack` und `xmrsplayer`
/// zuletzt  OPUS            - bis `opus-pure` dazukam
/// ```
///
/// Jedes Mal stand hier ein Test, der die Grenze festhielt, und jedes Mal
/// war er **einen Tag spaeter falsch**. Ein Test, der eine Grenze
/// festhaelt, muss mit ihr verschwinden; sonst verteidigt er sie gegen
/// die Messung.
///
/// Deshalb prueft er jetzt die andere Richtung: dass die sechs Formate
/// aus pygames Beispieldaten **ankommen**. Uebrig bleibt als Grenze nur
/// noch, was gar kein Klang ist - und dafuer gibt es die Meldung
/// darunter.
#[test]
fn die_beispieldateien_kommen_alle_an() {
    mit_mixer(|| {
        for datei in [
            "house_lo.wav",
            "house_lo.ogg",
            "house_lo.flac",
            "house_lo.mp3",
            "house_lo.wv",
            "house_lo.opus",
            "surfonasinewave.xm",
        ] {
            let Some(pfad) = pygame_beispiel(datei) else {
                continue;
            };
            let k = Sound::new(&pfad)
                .unwrap_or_else(|e| panic!("{datei} liess sich nicht lesen: {e}"));
            assert!(
                k.get_length() > 0.0,
                "{datei} kam mit der Laenge null an"
            );
        }
    });
}

/// Was **kein** Klang ist, sagt es sauber.
///
/// Kein Absturz, keine stille Stille - eine Meldung mit pygames Wortlaut.
#[test]
fn was_kein_klang_ist_sagt_es_sauber() {
    mit_mixer(|| {
        let f = Sound::from_bytes(b"nicht im Entferntesten ein Klang").unwrap_err();
        let t = f.to_string();
        assert!(
            // Der Satz ist **englisch**, seit alle Fehlertexte es sind:
            // ein Fehlertext ist das, was ein Mensch zu sehen bekommt.
            t.contains("Unrecognized audio format") || t.contains("12 bytes"),
            "{t}"
        );
    });
}

/// FLAC und MP3 kommen auf **dieselbe Byte-Zahl wie pygame**.
///
/// Die Probe darauf, dass die neuen Dekoder nicht nur irgendetwas liefern:
/// claxon und symphonia rechnen auf dieselben Abtastwerte wie SDL_mixer.
/// Gemessen an echtem pygame-ce 2.5.7:
///
/// ```text
/// house_lo.flac  1_253_296 Bytes, 7.1049 s   (wie .wav und .ogg)
/// house_lo.mp3   1_281_024 Bytes, 7.2620 s   (MP3 hat Vorlauf)
/// metadata.mp3         176 Bytes, 0.0010 s
/// ```
#[test]
fn flac_und_mp3_treffen_pygame() {
    mit_mixer(|| {
        for (datei, roh) in [
            ("house_lo.flac", 1_253_296usize),
            ("house_lo.mp3", 1_281_024),
            ("metadata.mp3", 176),
        ] {
            let Some(pfad) = pygame_beispiel(datei) else {
                continue;
            };
            let s = Sound::new(&pfad).unwrap_or_else(|e| panic!("{datei}: {e}"));
            assert_eq!(s.get_raw().len(), roh, "{datei}: Bytes");
        }
    });
}

/// Die Etiketten aus `metadata.mp3` - Wort fuer Wort wie bei pygame.
///
/// Die Datei hat eine **falsche** ID3-Laengenangabe; wer ihr glaubt, liest
/// weder die Etiketten noch den Ton. Siehe `rustgame_mixer::id3`.
#[test]
fn id3_etiketten_wie_pygame() {
    mit_mixer(|| {
        let Some(pfad) = pygame_beispiel("metadata.mp3") else {
            return;
        };
        rustgame_mixer::music::load(&pfad).unwrap();
        let m = rustgame_mixer::music::get_metadata().unwrap();
        assert_eq!(m.title, "Small Tone");
        assert_eq!(m.album, "Tones");
        assert_eq!(m.artist, "Audacity Generator");
        assert_eq!(m.copyright, "2023 Nobody");
    });
}

/// Unbrauchbare MP3-Daten geben einen Fehler, keine Panik.
///
/// Der Test hiess einmal "MP3 gibt einen Fehler" und pruefte, dass das
/// Format abgelehnt wird. Jetzt wird es gelesen - was bleibt, ist die
/// Zusage, dass ein kaputter Strom den Dekoder nicht umwirft.
#[test]
fn unbrauchbares_mp3_gibt_einen_fehler_und_keine_panik() {
    mit_mixer(|| {
        // Ein ID3-Kopf, wie ihn jede MP3 aus freier Wildbahn traegt.
        let mut daten = b"ID3\x04\x00\x00\x00\x00\x00\x00".to_vec();
        daten.extend_from_slice(&[0u8; 64]);
        // Auf dem Weg von `music` nennt der Dekodierer sich selbst ...
        let f = Sound::from_bytes_als_musik(&daten).unwrap_err();
        assert!(matches!(f, Fehler::Format(_)), "{f}");
        assert!(f.to_string().contains("MP3"));
        // ... auf dem Weg von `Sound` sagt pygame immer denselben Satz.
        let f = Sound::from_bytes(&daten).unwrap_err();
        assert_eq!(f.to_string(), "Unrecognized audio format");

        // Und ein blosser MPEG-Rahmenkopf ohne ID3 ebenso.
        let roh = [0xFFu8, 0xFB, 0x90, 0x00, 0, 0, 0, 0, 0, 0, 0, 0, 0];
        assert!(matches!(
            Sound::from_bytes(&roh).unwrap_err(),
            Fehler::Format(_)
        ));
    });
}

#[test]
fn unsinn_gibt_pygames_meldung() {
    mit_mixer(|| {
        // Gemessen: pygame sagt woertlich
        // "Couldn't read first 12 bytes of audio data".
        assert_eq!(
            Sound::from_bytes(b"nonsens").unwrap_err().to_string(),
            "Couldn't read first 12 bytes of audio data"
        );
        // Zwoelf Bytes, aber kein bekanntes Format.
        assert!(matches!(
            Sound::from_bytes(b"absolut kein klang").unwrap_err(),
            Fehler::Format(_)
        ));
        // Eine fehlende Datei.
        assert!(matches!(
            Sound::new("gibt-es-ganz-sicher-nicht.wav").unwrap_err(),
            Fehler::Datei(_)
        ));
    });
}

// ---------------------------------------------------------------------------
//  Die Bedingung, um die es geht
// ---------------------------------------------------------------------------

#[test]
fn alles_geht_auch_ohne_tongeraet() {
    // Der stille Treiber ist hier ohnehin gesetzt. Dieser Test haelt
    // fest, was das bedeuten soll: die gesamte Buchfuehrung laeuft, und
    // zwar zeitgetreu.
    mit_mixer(|| {
        assert_eq!(mixer::get_driver().unwrap(), "dummy");

        let s = synth::sound("explode").unwrap();
        assert!(s.get_length() > 0.2);
        assert_eq!(s.get_raw().len(), s.rahmen() * 4);

        s.set_volume(0.5);
        let c = s.play(0, 0, 0).unwrap();
        assert!(c.get_busy());
        assert_eq!(s.get_num_channels(), 1);
        assert_eq!(s.get_volume(), 0.5);

        // Zeitgetreu: nach der Laenge des Klangs ist Schluss, nicht
        // frueher und nicht spaeter.
        let gelaufen = warten(Duration::from_millis(900), || !c.get_busy());
        let soll = (s.get_length() * 1000.0) as u64;
        assert!(
            gelaufen.as_millis() as u64 + 5 >= soll,
            "zu frueh still: {} < {soll}",
            gelaufen.as_millis()
        );
        assert!(
            (gelaufen.as_millis() as u64) < soll + 150,
            "zu spaet still: {} > {soll}",
            gelaufen.as_millis()
        );
        assert!(!mixer::get_busy());
        assert_eq!(s.get_num_channels(), 0);
    });
}

/// **MS-ADPCM, Byte fuer Byte wie SDL** - gemessen ohne Umrechnung.
///
/// `examples/data/secosmic_lo.wav` ist Art 2 (Microsoft ADPCM), mono,
/// 11025 Hz, Bloecke zu 256 Bytes mit 500 Werten. Unter einem Mixer in
/// **genau diesem** Format faellt das Umrechnen weg, und dann geben beide
/// Engines dieselben 72.000 Bytes - derselbe SHA-256.
///
/// Das Umrechnen selbst weicht ab (benannte Grenze, siehe
/// `vergleich_module.py`), deshalb wird hier bewusst in der Quellrate
/// geprueft: sonst misst der Test zwei Dinge auf einmal und sagt bei
/// einem Fehlschlag nicht, welches.
///
/// Die Werte stammen aus echtem pygame-ce 2.5.7:
///
/// ```text
/// mixer.init(frequency=11025, size=-16, channels=1)
/// len(Sound("secosmic_lo.wav").get_raw())  ->  72000
/// get_length()                             ->  3.2653
/// ```
#[test]
fn ms_adpcm_trifft_pygame_byteweise() {
    vorbereiten();
    mixer::init(Einstellungen {
        frequency: 11_025,
        channels: 1,
        ..Default::default()
    })
    .expect("der stille Treiber geht immer");
    if let Some(pfad) = pygame_beispiel("secosmic_lo.wav") {
        let s = Sound::new(&pfad).unwrap_or_else(|e| panic!("ADPCM: {e}"));
        let roh = s.get_raw();
        assert_eq!(roh.len(), 72_000, "Bytes");
        // Ein paar Werte aus der Mitte, an echtem pygame abgelesen -
        // die Laenge allein wuerde auch ein falscher Dekodierer treffen.
        let wert = |n: usize| i16::from_le_bytes([roh[n * 2], roh[n * 2 + 1]]);
        assert_eq!(
            (wert(0), wert(1), wert(500), wert(501), wert(20_000)),
            (0, 0, -220, -293, 9404),
            "Abtastwerte"
        );
        assert!(
            (s.get_length() - 3.2653).abs() < 0.001,
            "Laenge {}",
            s.get_length()
        );
    }
    mixer::quit();
}

/// **Der Umrechner trifft SDL Wert fuer Wert** - die eigentliche Probe.
///
/// `house_lo.wav` ist 22050 Hz Mono und wird unter der Vorgabe
/// (44100 Hz, Stereo) auf beides umgerechnet. Bis zum 29.09.2026 stand
/// hier eine lineare Interpolation; die traf die **Laenge** genau und die
/// Werte nicht:
///
/// ```text
/// rustgame (linear): -3584, -3584, -3712, -3712, -3840, -3840
/// pygame   (SDL):    -3578, -3578, -4150, -4150, -4388, -4388
/// ```
///
/// Seit der Uebertragung von `SDL_ResampleAudio` samt Filtertabelle
/// stimmen sie. Die Werte hier stammen aus echtem pygame-ce 2.5.7; der
/// vollstaendige Vergleich (SHA-256 ueber alle 1.253.296 Bytes) steht in
/// `vergleich_module.py`.
#[test]
fn umrechner_trifft_sdl_wert_fuer_wert() {
    mit_mixer(|| {
        let Some(pfad) = pygame_beispiel("house_lo.wav") else {
            return;
        };
        let s = Sound::new(&pfad).unwrap_or_else(|e| panic!("{e}"));
        let roh = s.get_raw();
        assert_eq!(roh.len(), 1_253_296, "Bytes");
        let wert = |n: usize| i16::from_le_bytes([roh[n * 2], roh[n * 2 + 1]]);
        assert_eq!(
            [wert(0), wert(1), wert(2), wert(3), wert(4), wert(5), wert(6), wert(7)],
            [-3578, -3578, -4150, -4150, -4388, -4388, -4339, -4339],
            "der Anfang - hier war die lineare Fassung als Erstes daneben"
        );
        assert_eq!(
            [wert(1_000), wert(100_000), wert(313_000)],
            [5115, 9726, -2301],
            "Stichproben aus der Mitte"
        );
        assert!(
            (s.get_length() - 7.104_853).abs() < 1e-5,
            "Laenge {}",
            s.get_length()
        );
    });
}
