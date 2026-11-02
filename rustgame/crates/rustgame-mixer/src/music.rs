//! music.rs
//! ========
//! `pygame.mixer.music` - die Musikspur.
//!
//! Eine einzige Spur, an den Mischkanaelen vorbei, mit eigener
//! Lautstaerke und eigener Warteschlange. `mixer.stop()` und
//! `mixer.pause()` fassen sie nicht an; dafuer gibt es [`stop`] und
//! [`pause`] hier.
//!
//! ## Drei Dinge, die anders sind als man denkt
//!
//! **1. `loops` zaehlt anders als bei [`crate::Sound`].** Gemessen mit
//! einem 0.1 Sekunden langen Stueck:
//!
//! | `loops` | `Sound.play` | `music.play` |
//! |---|---|---|
//! | 0 | 1x | 1x |
//! | 1 | **2x** | **1x** |
//! | 2 | 3x | 2x |
//! | 5 | 6x | 5x |
//!
//! Bei der Musik ist `loops` also die Zahl der **Durchlaeufe**, und `0`
//! und `1` bedeuten dasselbe. Der Grund steht in SDL_mixers
//! `Mix_FadeInMusicPos`: `if (loops == 0) loops = 1;`.
//!
//! **2. Die Lautstaerke steht nach `init()` nicht auf 1.0.** Gemessen:
//! `music.get_volume()` liefert `0.9921875`, also 127 von 128 Stufen.
//! pygames `_init` ruft zum Schluss `Mix_VolumeMusic(127)` - eine
//! einzelne Stufe unter voll, ohne erkennbaren Grund.
//!
//! **3. `get_pos()` ist keine Stelle im Stueck, sondern eine Stoppuhr.**
//! Sie zaehlt, wie lange seit `play()` gespielt wurde - Pausen nicht
//! mitgerechnet. Gemessen springt sie weder bei [`rewind`] noch bei
//! [`set_pos`] zurueck: nach `play(); sleep(0.05); rewind()` liefert sie
//! `59`, nicht `0`. Das ist so nachgebaut.

use crate::konstanten::*;
use crate::sound::Sound;
use crate::treiber::Stimme;
use crate::zustand::{self, Lauf};
use crate::{Ergebnis, Fehler};

/// Laedt ein Stueck - `pygame.mixer.music.load(dateiname)`.
///
/// Wirft weg, was vorher geladen war, **und** was in der Warteschlange
/// stand - gemessen, und in `src_c/music.c` ausdruecklich so geschrieben.
pub fn load(pfad: impl AsRef<std::path::Path>) -> Ergebnis<()> {
    let pfad = pfad.as_ref();
    // Die Datei wird **hier** gelesen, nicht in `Sound::new`, weil die
    // Etiketten in den Rohbytes stehen und nicht in den Abtastwerten.
    // Zweimal lesen waere die Alternative gewesen.
    let daten = crate::sound::datei_lesen(pfad)?;
    load_bytes(&daten)
}

/// Laedt ein Stueck aus einem Byte-Puffer.
///
/// Das Gegenstueck zu `music.load(io.BytesIO(daten))`.
pub fn load_bytes(daten: &[u8]) -> Ergebnis<()> {
    let stueck = Sound::from_bytes_als_musik(daten)?;
    let etiketten = crate::id3::etiketten(daten);
    setzen(stueck)?;
    zustand::mit(|z| {
        z.musik.etiketten = etiketten;
        Ok(())
    })
}

/// Nimmt einen bereits gelesenen Klang als Musik.
///
/// Es gibt in pygame keine Entsprechung - dort ist Musik immer eine
/// Datei. Hier ist es der bequeme Weg fuer selbst erzeugte Stuecke, und
/// er kostet nichts: intern ist ein Musikstueck ohnehin dasselbe wie ein
/// [`Sound`], nur an einer anderen Spur.
pub fn load_sound(stueck: &Sound) -> Ergebnis<()> {
    setzen(stueck.clone())
}

fn setzen(stueck: Sound) -> Ergebnis<()> {
    zustand::mit(|z| {
        z.pruefen()?;
        if z.musik.lauf.is_some() {
            if let Some(t) = z.treiber.as_mut() {
                t.stoppen(Stimme::Musik);
            }
        }
        z.musik.lauf = None;
        z.musik.schlange = None;
        z.musik.geladen = Some(stueck);
        Ok(())
    })
}

/// Wirft das geladene Stueck weg - `music.unload()`.
///
/// Gemessen: danach wirft `play()` wieder `music not loaded`.
pub fn unload() -> Ergebnis<()> {
    zustand::mit(|z| {
        z.pruefen()?;
        if let Some(t) = z.treiber.as_mut() {
            t.stoppen(Stimme::Musik);
        }
        z.musik.geladen = None;
        z.musik.schlange = None;
        z.musik.lauf = None;
        z.musik.etiketten = Metadaten::default();
        Ok(())
    })
}

/// Haengt ein Stueck hinten an - `music.queue(dateiname, loops=...)`.
pub fn queue(pfad: impl AsRef<std::path::Path>, loops: i32) -> Ergebnis<()> {
    let stueck = Sound::from_bytes_als_musik(&crate::sound::datei_lesen(pfad.as_ref())?)?;
    queue_sound(&stueck, loops)
}

/// Haengt einen bereits gelesenen Klang hinten an.
pub fn queue_sound(stueck: &Sound, loops: i32) -> Ergebnis<()> {
    zustand::mit(|z| {
        z.pruefen()?;
        z.musik.schlange = Some((stueck.clone(), durchlaeufe(loops)));
        Ok(())
    })
}

/// Spielt das geladene Stueck - `music.play(loops, start, fade_ms)`.
///
/// `loops` zaehlt die **Durchlaeufe**, nicht die Wiederholungen: `0` und
/// `1` spielen beide genau einmal, `-1` ist endlos. Siehe die Tabelle im
/// Modulkopf.
///
/// `start` ist die Stelle in **Sekunden**, an der begonnen wird.
///
/// **Eine bewusste Abweichung:** gemessen hat echtes pygame `start` bei
/// einer WAV-Datei schlicht ueberschlagen - `play(0, start=0.5)` auf ein
/// Stueck von einer Sekunde lief die vollen 0.976 s, nicht die halbe
/// Zeit. Das haengt an SDL_mixers WAV-Dekoder und ist offensichtlich
/// nicht das, was `start` bedeuten soll. Hier faengt das Stueck wirklich
/// an der genannten Stelle an.
///
/// Gemessen: ohne vorheriges [`load`] kommt `music not loaded`.
pub fn play(loops: i32, start: f64, fade_ms: i32) -> Ergebnis<()> {
    zustand::mit(|z| {
        z.pruefen()?;
        let Some(stueck) = z.musik.geladen.clone() else {
            return Err(Fehler::NichtGeladen);
        };
        let ab_ms = (start.max(0.0) * 1000.0) as u64;
        z.musik.lauf = Some(Lauf {
            start: rustgame_core::time::get_ticks(),
            pause_seit: None,
            pause_summe: 0,
            rahmen: stueck.rahmen() as u64,
            frequenz: stueck.frequenz(),
            durchlaeufe: durchlaeufe(loops),
            ab_ms,
            maxtime: None,
            ausblenden: None,
            pos_versatz: 0,
        });
        z.musik_beauftragen(ab_ms, fade_ms.max(0) as u64);
        Ok(())
    })
}

/// Faengt das laufende Stueck von vorn an - `music.rewind()`.
///
/// Gemessen: [`get_pos`] springt dabei **nicht** zurueck, sondern zaehlt
/// weiter. pygame setzt nur SDLs Stroemung zurueck, nicht seinen eigenen
/// Zaehler.
pub fn rewind() -> Ergebnis<()> {
    zustand::mit(|z| {
        z.pruefen()?;
        springen(z, 0)
    })
}

/// Springt an eine Stelle - `music.set_pos(sekunden)`.
///
/// Gemessen: laeuft gerade nichts, kommt `Music isn't playing`. Und wie
/// bei [`rewind`] zaehlt [`get_pos`] danach einfach weiter, statt auf die
/// neue Stelle zu springen.
pub fn set_pos(sekunden: f64) -> Ergebnis<()> {
    zustand::mit(|z| {
        z.pruefen()?;
        if z.musik.lauf.is_none() {
            return Err(Fehler::SpieltNicht);
        }
        springen(z, (sekunden.max(0.0) * 1000.0) as u64)
    })
}

/// Der gemeinsame Kern von [`rewind`] und [`set_pos`].
fn springen(z: &mut zustand::Zustand, ab_ms: u64) -> Ergebnis<()> {
    let jetzt = rustgame_core::time::get_ticks();
    let Some(alt) = z.musik.lauf.as_ref() else {
        return Ok(());
    };
    // Was `get_pos` bisher gezaehlt hat, wird mitgenommen - genau das
    // ist der gemessene Unterschied zu "auf null setzen".
    let bisher = alt.pos_versatz + alt.verstrichen(jetzt);
    let durchlaeufe = alt.durchlaeufe;
    let rahmen = alt.rahmen;
    let frequenz = alt.frequenz;
    z.musik.lauf = Some(Lauf {
        start: jetzt,
        pause_seit: None,
        pause_summe: 0,
        rahmen,
        frequenz,
        durchlaeufe,
        ab_ms,
        maxtime: None,
        ausblenden: None,
        pos_versatz: bisher,
    });
    z.musik_beauftragen(ab_ms, 0);
    Ok(())
}

/// Haelt an - `music.stop()`.
///
/// Gemessen: die Warteschlange wird dabei **weggeworfen**, nicht
/// gestartet - anders als bei [`crate::Channel::stop`]. pygames
/// `music_stop` befreit sie ausdruecklich vorher ("To prevent the
/// queue_music from playing"). Das Endereignis faellt trotzdem an.
pub fn stop() -> Ergebnis<()> {
    zustand::mit(|z| {
        z.pruefen()?;
        let jetzt = rustgame_core::time::get_ticks();
        z.musik_beenden(false, jetzt);
        Ok(())
    })
}

/// Haelt an, ohne die Stelle zu verlieren - `music.pause()`.
pub fn pause() -> Ergebnis<()> {
    zustand::mit(|z| {
        z.pruefen()?;
        let jetzt = rustgame_core::time::get_ticks();
        if let Some(l) = z.musik.lauf.as_mut() {
            if l.pause_seit.is_none() {
                l.pause_seit = Some(jetzt);
                if let Some(t) = z.treiber.as_mut() {
                    t.pausieren(Stimme::Musik);
                }
            }
        }
        Ok(())
    })
}

/// Macht weiter - `music.unpause()`.
pub fn unpause() -> Ergebnis<()> {
    zustand::mit(|z| {
        z.pruefen()?;
        let jetzt = rustgame_core::time::get_ticks();
        if let Some(l) = z.musik.lauf.as_mut() {
            if let Some(seit) = l.pause_seit {
                l.pause_summe += jetzt.saturating_sub(seit);
                l.pause_seit = None;
                if let Some(t) = z.treiber.as_mut() {
                    t.fortsetzen(Stimme::Musik);
                }
            }
        }
        Ok(())
    })
}

/// Blendet aus und hoert dann auf - `music.fadeout(ms)`.
///
/// Wie [`stop`] wirft es die Warteschlange weg.
pub fn fadeout(millisekunden: i32) -> Ergebnis<()> {
    zustand::mit(|z| {
        z.pruefen()?;
        let jetzt = rustgame_core::time::get_ticks();
        z.musik.schlange = None;
        if millisekunden <= 0 {
            z.musik_beenden(false, jetzt);
            return Ok(());
        }
        if let Some(l) = z.musik.lauf.as_mut() {
            let ab = l.verstrichen(jetzt);
            l.ausblenden = Some((ab, millisekunden as u64));
        }
        Ok(())
    })
}

/// Laeuft gerade Musik - `music.get_busy()`.
///
/// Gemessen: ein **pausiertes** Stueck gilt hier als **nicht** belegt -
/// genau umgekehrt zu [`crate::Channel::get_busy`], wo eine Pause als
/// belegt zaehlt. pygames `music_get_busy` prueft ausdruecklich
/// `Mix_PlayingMusic() && !Mix_PausedMusic()`.
pub fn get_busy() -> bool {
    zustand::mit(|z| {
        z.offen.is_some()
            && z.musik
                .lauf
                .as_ref()
                .is_some_and(|l| l.pause_seit.is_none())
    })
}

/// Wie lange schon gespielt wird, in Millisekunden - `music.get_pos()`.
///
/// Gemessen: `-1`, solange noch nie gespielt wurde. Danach eine
/// Stoppuhr, die waehrend einer Pause stehen bleibt und bei [`rewind`]
/// oder [`set_pos`] **nicht** zurueckspringt.
///
/// **Eine bewusste Abweichung:** nach [`stop`] gibt echtes pygame hier
/// keinen sinnvollen Wert mehr, sondern eine grosse Zahl - sein
/// `music_pos_time` ist ein `Uint64`, auf `-1` gesetzt, und der Test
/// `music_pos_time < 0` kann fuer eine vorzeichenlose Zahl nie zutreffen.
/// Gemessen kam dort `1011` heraus, zusammengesetzt aus der gespielten
/// Zeit und der Laufzeit des Programms. Diese Kiste liefert stattdessen
/// wieder `-1`, also das, was die Doku verspricht.
pub fn get_pos() -> i64 {
    zustand::mit(|z| {
        if z.offen.is_none() {
            return -1;
        }
        let jetzt = rustgame_core::time::get_ticks();
        match z.musik.lauf.as_ref() {
            Some(l) => (l.pos_versatz + l.verstrichen(jetzt)) as i64,
            None => -1,
        }
    })
}

/// Die Lautstaerke von 0.0 bis 1.0 - `music.get_volume()`.
///
/// Gemessen: direkt nach `mixer.init()` ist das `0.9921875`, nicht `1.0`.
pub fn get_volume() -> f64 {
    zustand::mit(|z| z.musik.lautstaerke as f64 / MAX_VOLUME as f64)
}

/// Stellt die Lautstaerke ein - `music.set_volume(wert)`.
///
/// 128 Stufen, ueber 1.0 wird gekappt, negativ aendert nichts - wie
/// ueberall.
pub fn set_volume(wert: f64) -> Ergebnis<()> {
    zustand::mit(|z| {
        z.pruefen()?;
        z.musik.lautstaerke = crate::mixer::stufe(wert, z.musik.lautstaerke);
        let f = z.musik.lautstaerke as f32 / MAX_VOLUME as f32;
        if let Some(t) = z.treiber.as_mut() {
            t.lautstaerke(Stimme::Musik, f);
        }
        Ok(())
    })
}

/// Welches Ereignis das Ende melden soll - `music.set_endevent()`.
///
/// [`crate::konstanten::NOEVENT`] schaltet es ab. Anders als beim Kanal
/// traegt das Ereignis **kein** `code`-Feld - gemessen kam dort ein
/// leeres Ereignis heraus.
pub fn set_endevent(typ: u32) {
    zustand::mit(|z| z.musik.endevent = typ);
}

/// Welches Ereignis gesetzt ist - `music.get_endevent()`.
pub fn get_endevent() -> u32 {
    zustand::mit(|z| z.musik.endevent)
}

/// Was an Beschriftung im Stueck steht - `music.get_metadata()`.
///
/// Gemessen liefert pygame immer vier Schluessel - `title`, `album`,
/// `artist`, `copyright` - und fuer eine WAV-Datei sind alle vier leer.
/// Genau das gibt diese Kiste zurueck: WAV traegt keine solchen Angaben,
/// und OGG-Kommentare liest rodio nicht heraus. Die vier Felder sind
/// trotzdem da, damit Programme, die sie abfragen, nicht umfallen.
///
/// Gemessen: ohne geladenes Stueck kommt `music not loaded`.
pub fn get_metadata() -> Ergebnis<Metadaten> {
    zustand::mit(|z| {
        z.pruefen()?;
        if z.musik.geladen.is_none() {
            return Err(Fehler::NichtGeladen);
        }
        Ok(z.musik.etiketten.clone())
    })
}

/// Die Etiketten **fremder Daten**, ohne sie zu laden.
///
/// Gemessen: `music.get_metadata(pfad)` geht auch nach `unload()`, und
/// `mixer_music_test.test_music_get_metadata` prueft beide Wege gegen
/// dasselbe Woerterbuch. Wer nur den ersten baut, hat eine Funktion, die
/// bei jeder zweiten Zeile des Tests einen `TypeError` gibt.
pub fn metadaten_von(daten: &[u8]) -> Ergebnis<Metadaten> {
    zustand::mit(|z| z.pruefen())?;
    // **Was kein Klang ist, hat keine Etiketten.** Hier kam fuer jede
    // Datei ein Satz leerer Felder zurueck, auch fuer eine, die gar kein
    // Tonformat ist; pygame gibt dort `Unrecognized audio format` - es
    // reicht die Daten an alle Dekodierer und keiner will sie. Gefunden
    // hat das die Probe `Fehlertexte`.
    crate::sound::dekodierer_pruefen(daten)?;
    Ok(crate::id3::etiketten(daten))
}

/// Die vier Felder, die `music.get_metadata()` immer liefert.
///
/// Zweitname fuer [`crate::id3::Etiketten`]: dort werden sie gelesen, hier
/// werden sie ausgegeben, und eine zweite Struktur haette nur Umpacken
/// bedeutet.
pub type Metadaten = crate::id3::Etiketten;

/// Rechnet pygames `loops` fuer die Musik in Durchlaeufe um.
///
/// Gemessen: `0` und `1` sind beide **ein** Durchlauf, `-1` ist endlos.
fn durchlaeufe(loops: i32) -> i32 {
    if loops < 0 {
        -1
    } else {
        loops.max(1)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn loops_zaehlen_wie_gemessen() {
        assert_eq!(durchlaeufe(0), 1);
        assert_eq!(durchlaeufe(1), 1);
        assert_eq!(durchlaeufe(2), 2);
        assert_eq!(durchlaeufe(5), 5);
        assert_eq!(durchlaeufe(-1), -1);
    }
}
