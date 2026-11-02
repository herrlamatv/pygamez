//! mixer.rs
//! ========
//! Die Modulfunktionen von `pygame.mixer`.
//!
//! ## `init()` und der Rechner ohne Soundkarte
//!
//! Das ist die entscheidende Stelle dieser Kiste. In pygame ist
//! `mixer.init()` eine Huerde: geht sie nicht, geht gar nichts, und jedes
//! `Sound(...)` wirft danach `mixer not initialized`.
//!
//! Hier nicht. [`init`] gibt zwar einen sauberen
//! [`crate::Fehler::KeinGeraet`] zurueck, wenn kein Tongeraet da ist -
//! **aber der Mixer laeuft danach trotzdem**, still. `get_init()` meldet
//! die Einstellungen, [`crate::Sound`] und [`crate::Channel`] lassen sich
//! anlegen, befuellen und abfragen, `get_busy()` und `music.get_pos()`
//! liefern zeitgetreue Zahlen. Nur zu hoeren ist nichts.
//!
//! Das ist der einzige bewusste Bruch mit pygame in dieser Kiste, und er
//! ist Absicht: ein Spiel soll auf einem Rechner ohne Soundkarte laufen,
//! nicht abstuerzen und auch nicht jeden Tonaufruf einzeln absichern
//! muessen. Wer den Unterschied sehen will, fragt den Rueckgabewert von
//! [`init`] oder [`get_driver`].
//!
//! ## Was `init()` aus den Wuenschen macht
//!
//! Gemessen und aus `src_c/mixer.c` nachgelesen:
//!
//! * `frequency`, `size`, `channels`, `buffer` gleich `0` heissen
//!   "nimm, was `pre_init` gesetzt hat" - und ohne `pre_init` die
//!   Vorgaben `44100, -16, 2, 512`.
//! * `channels` wird auf 1, 2, 4 oder 6 **gerundet**: 3 wird 2, 5 wird 4,
//!   alles ueber 5 wird 6.
//! * `size = 32` wird zu **-32**, weil daraus `AUDIO_F32SYS` wird und
//!   `get_init()` das Vorzeichenbit zurueckliest. `size = 8` bleibt `8`.
//! * `buffer` wird auf die naechste Zweierpotenz aufgerundet, mindestens
//!   256.

use crate::channel::Channel;
use crate::konstanten::*;
use crate::treiber::{self, Stimme, Wahl};
use crate::zustand::{self, Spec};
use crate::{Ergebnis, Fehler};

/// Was `init()` haben moechte - die Argumente von `pygame.mixer.init()`.
///
/// `Default` ist genau `init()` ohne Argumente: jedes Feld `0`, also
/// "nimm die Vorgabe oder was `pre_init` gesagt hat".
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Einstellungen {
    /// Abtastrate in Hertz; `0` heisst Vorgabe.
    pub frequency: i32,
    /// Bits je Wert, negativ heisst vorzeichenbehaftet; `0` heisst Vorgabe.
    pub size: i32,
    /// Ausgabespuren; `0` heisst Vorgabe.
    pub channels: i32,
    /// Puffergroesse in Rahmen; `0` heisst Vorgabe.
    pub buffer: i32,
}

impl Einstellungen {
    /// Kurzform fuer alle vier Werte auf einmal.
    pub fn neu(frequency: i32, size: i32, channels: i32, buffer: i32) -> Einstellungen {
        Einstellungen {
            frequency,
            size,
            channels,
            buffer,
        }
    }
}

// ---------------------------------------------------------------------------
//  Anlauf und Ende
// ---------------------------------------------------------------------------

/// Legt fest, was ein spaeteres [`init`] nehmen soll -
/// `pygame.mixer.pre_init()`.
///
/// Aendert am laufenden Mixer nichts; gemessen bleibt `get_init()` danach
/// unveraendert. Ein Feld auf `0` bedeutet hier "zurueck auf die
/// Vorgabe", nicht "unveraendert lassen" - gemessen: nach
/// `pre_init(0, 0, 0, 0)` liefert `init()` wieder `(44100, -16, 2)`.
pub fn pre_init(wunsch: Einstellungen) {
    zustand::mit(|z| {
        z.wunsch = Einstellungen {
            frequency: if wunsch.frequency != 0 {
                wunsch.frequency
            } else {
                VORGABE_FREQUENZ
            },
            size: if wunsch.size != 0 {
                wunsch.size
            } else {
                VORGABE_GROESSE
            },
            channels: if wunsch.channels != 0 {
                wunsch.channels
            } else {
                VORGABE_KANAELE
            },
            buffer: if wunsch.buffer != 0 {
                wunsch.buffer
            } else {
                VORGABE_PUFFER
            },
        };
    });
}

/// Faehrt den Mixer hoch - `pygame.mixer.init()`.
///
/// Gemessen: ein zweites `init()` bei laufendem Mixer aendert nichts und
/// beschwert sich nicht.
///
/// Gibt [`crate::Fehler::KeinGeraet`] zurueck, wenn kein Tongeraet da
/// ist - **der Mixer laeuft danach trotzdem, nur still.** Siehe oben.
pub fn init(wunsch: Einstellungen) -> Ergebnis<()> {
    let spec = auswerten(wunsch, zustand::mit(|z| z.wunsch))?;

    zustand::mit(|z| {
        if z.offen.is_some() {
            return Ok(());
        }
        let (treiber, fehler) = match treiber::oeffnen(z.wahl) {
            Ok(t) => (t, None),
            Err(e) => (
                Box::new(treiber::StummerTreiber::neu()) as Box<dyn treiber::Treiber>,
                Some(e),
            ),
        };
        z.treiber = Some(treiber);
        z.offen = Some(spec);
        z.kanaele_setzen(VORGABE_STIMMEN);
        // Gemessen: `music.get_volume()` steht direkt nach `init()` auf
        // 0.9921875, nicht auf 1.0 - pygames `_init` ruft am Ende
        // `Mix_VolumeMusic(127)`. Ein Wert, ueber den man ewig raetselt,
        // wenn man ihn nicht kennt.
        z.musik.lautstaerke = MAX_VOLUME - 1;
        match fehler {
            Some(e) => Err(e),
            None => Ok(()),
        }
    })
}

/// Faehrt den Mixer herunter - `pygame.mixer.quit()`.
///
/// Gemessen: geht auch, wenn gar nichts lief, und beschwert sich nicht.
/// Bereits angelegte [`crate::Sound`] ueberleben es - sie behalten ihre
/// Daten und lassen sich nach einem neuen `init()` weiter benutzen.
pub fn quit() {
    zustand::mit(|z| {
        for i in 0..z.anzahl {
            if let Some(t) = z.treiber.as_mut() {
                t.stoppen(Stimme::Kanal(i));
            }
        }
        if let Some(t) = z.treiber.as_mut() {
            t.stoppen(Stimme::Musik);
        }
        zustand::zuruecksetzen(z);
    });
}

/// Womit der Mixer laeuft - `pygame.mixer.get_init()`.
///
/// Liefert `(Frequenz, Groesse, Spuren)`; gemessen `None` vor `init()`
/// und `(44100, -16, 2)` danach. Die Puffergroesse verraet pygame hier
/// **nicht** - wer sie braucht, nimmt [`get_spec`].
pub fn get_init() -> Option<(i32, i32, i32)> {
    zustand::mit(|z| z.offen.map(|s| (s.frequenz, s.groesse, s.spuren)))
}

/// Wie [`get_init`], aber mit der Puffergroesse.
pub fn get_spec() -> Option<Spec> {
    zustand::mit(|z| z.offen)
}

/// Wie der Treiber heisst - `pygame.mixer.get_driver()`.
///
/// `"dummy"` heisst: es ist nichts zu hoeren.
pub fn get_driver() -> Ergebnis<String> {
    zustand::mit(|z| {
        z.pruefen()?;
        Ok(z.treiber
            .as_ref()
            .map_or_else(|| "unknown".to_string(), |t| t.name().to_string()))
    })
}

/// Welcher Treiber genommen werden soll.
///
/// Wirkt erst beim naechsten [`init`] - genau wie
/// `rustgame_display::display::set_treiber`.
pub fn set_treiber(wahl: Wahl) {
    zustand::mit(|z| z.wahl = wahl);
}

/// Die Fassung der Tonbibliothek -
/// `pygame.mixer.get_sdl_mixer_version()`.
///
/// Gibt [`SDL_MIXER_VERSION`] zurueck, also die an echtem pygame-ce 2.5.7
/// gemessene `(2, 8, 1)`.
///
/// **Der Kopf dieses Vermerks sagte bis eben das Gegenteil** - "gibt
/// `None` zurueck, und das ist die einzig ehrliche Antwort" -, waehrend
/// der Rumpf darunter schon `Some(...)` lieferte und im Beiwerk
/// ausbuchstabierte, warum. Zwei Saetze, die einander widersprachen, und
/// der obere war der gut begruendete. Die Begruendung fuer die Zahl steht
/// gleich hier im Rumpf; sie gilt.
pub fn get_sdl_mixer_version() -> Option<(i32, i32, i32)> {
    // Hier stand `None`, mit der Begruendung: rustgame bindet kein
    // SDL_mixer ein, also waere jede Zahl erfunden. Das klang richtig
    // und war es nicht - **pygames eigene Testsuite hat es widerlegt.**
    //
    // `image_test.py` schreibt die Regel fuer den Schwesteraufruf aus:
    // `get_sdl_image_version()` ist `None` **genau dann**, wenn
    // `get_extended()` falsch ist - also wenn die erweiterten Formate
    // fehlen, nicht wenn die Bibliothek fehlt. Bei uns gehen sie, und
    // die Paarung widersprach sich damit in sich.
    //
    // Gemeldet wird deshalb die Fassung, deren **Leistungsumfang** hier
    // nachgebaut ist: die, die pygame-ce 2.5.7 mitbringt. Dieselbe
    // Entscheidung wie bei `version.vernum` (`2.5.7`) und bei
    // `image::get_sdl_image_version` - dort steht die lange Fassung der
    // Begruendung.
    //
    // Was wirklich fehlt, sagt nicht diese Zahl, sondern ein klarer
    // Fehler an der Stelle, an der es fehlt. Und der Weg dorthin ist
    // ohnehin ein anderer: rodio, ein eigener WAV-Dekoder und die aus
    // `mixer.c` uebertragenen Mischregeln.
    //
    // Nebenbei: `mixer_music_test.py` liess sich mit `None` nicht einmal
    // **laden** - es vergleicht die Fassung beim Import.
    Some(SDL_MIXER_VERSION)
}

/// Der Pfad zur Klangbank fuer MIDI - `mixer.get_soundfont`.
///
/// Anfangs `None`, genau wie in pygame.
pub fn get_soundfont() -> Option<String> {
    zustand::mit(|z| z.soundfont.clone())
}

/// Setzt den Pfad zur Klangbank - `mixer.set_soundfont`.
///
/// **Wirkt hier nichts**, und das steht auch so da: rustgame spielt kein
/// MIDI, also gibt es nichts, wofuer eine Klangbank gebraucht wuerde.
/// Der Pfad wird trotzdem gemerkt und von [`get_soundfont`] wieder
/// herausgegeben - pygame tut genau das, ohne die Datei zu pruefen
/// (gemessen: ein Pfad, den es gar nicht gibt, wird anstandslos
/// uebernommen). Ein Programm, das den Wert setzt und wieder liest,
/// verhaelt sich also gleich; nur klingen wird nichts.
pub fn set_soundfont(pfad: Option<&str>) {
    zustand::mit(|z| z.soundfont = pfad.map(|s| s.to_string()));
}

// ---------------------------------------------------------------------------
//  Kanaele
// ---------------------------------------------------------------------------

/// Wie viele Mischkanaele es gibt -
/// `pygame.mixer.get_num_channels()`.
///
/// Gemessen: nach `init()` sind es **8**.
pub fn get_num_channels() -> Ergebnis<usize> {
    zustand::mit(|z| {
        z.pruefen()?;
        Ok(z.anzahl)
    })
}

/// Legt die Kanalzahl fest - `pygame.mixer.set_num_channels()`.
///
/// Gemessen: wird verkleinert, verstummen die wegfallenden Kanaele
/// sofort. `set_num_channels(0)` ist erlaubt und laesst keinen uebrig.
pub fn set_num_channels(anzahl: usize) -> Ergebnis<()> {
    zustand::mit(|z| {
        z.pruefen()?;
        z.kanaele_setzen(anzahl);
        Ok(())
    })
}

/// Haelt die ersten `anzahl` Kanaele frei -
/// `pygame.mixer.set_reserved()`.
///
/// Reservierte Kanaele bekommt [`crate::Sound::play`] nicht mehr; sie
/// bleiben `Channel(0)`, `Channel(1)` und so weiter vorbehalten.
/// Gemessen: nach `set_reserved(2)` landet der erste `Sound.play()` auf
/// Kanal **2**. Der Rueckgabewert ist die Zahl, die wirklich reserviert
/// wurde - mehr als `get_num_channels()` geht nicht.
pub fn set_reserved(anzahl: usize) -> Ergebnis<usize> {
    zustand::mit(|z| {
        z.pruefen()?;
        z.reserviert = anzahl.min(z.anzahl);
        Ok(z.reserviert)
    })
}

/// Sucht einen freien Kanal - `pygame.mixer.find_channel()`.
///
/// Gemessen und ueberraschend: hier werden die **reservierten Kanaele
/// nicht uebersprungen**. Nach `set_reserved(2)` liefert `find_channel()`
/// weiter Kanal 0, waehrend `Sound.play()` bei Kanal 2 anfaengt. Das ist
/// kein Tippfehler dieser Kiste, sondern SDL_mixers `Mix_GroupAvailable`,
/// das bei null zu zaehlen beginnt.
///
/// `force` nimmt den am laengsten laufenden Kanal, wenn alle belegt sind.
/// Ohne `force` kommt `None`.
///
/// **Ein Sonderfall, der aus der Vorlage kommt:** gibt es gar keine
/// Kanaele (`set_num_channels(0)`), dann findet `force` auch keinen
/// aeltesten - und `mixer_find_channel` in `src_c/mixer.c` reicht die
/// `-1` von `Mix_GroupOldest` ungeprueft an `pgChannel_New` weiter, das
/// daraus `IndexError: invalid channel index` macht. Gemessen an echtem
/// pygame nach `set_num_channels(0)`:
///
/// ```text
/// find_channel()      ->  None
/// find_channel(True)  ->  IndexError: invalid channel index
/// ```
pub fn find_channel(force: bool) -> Ergebnis<Option<Channel>> {
    zustand::mit(|z| {
        z.pruefen()?;
        if let Some(i) = (0..z.anzahl).find(|&i| z.kanaele[i].lauf.is_none()) {
            return Ok(Some(Channel::roh(i)));
        }
        if !force {
            return Ok(None);
        }
        // Der aelteste - bei Gleichstand der mit dem hoeheren Index,
        // genau wie SDL_mixers `Mix_GroupOldest` mit seinem `<=`.
        let mut gewaehlt = None;
        let mut fruehestens = u64::MAX;
        for i in 0..z.anzahl {
            if let Some(l) = z.kanaele[i].lauf.as_ref() {
                if l.start <= fruehestens {
                    fruehestens = l.start;
                    gewaehlt = Some(i);
                }
            }
        }
        // Kein aeltester, obwohl `force` - also gibt es ueberhaupt keine
        // Kanaele. pygame kommt hier mit `-1` heraus; siehe oben.
        match gewaehlt {
            Some(i) => Ok(Some(Channel::roh(i))),
            None => Err(Fehler::UngueltigerKanal(usize::MAX)),
        }
    })
}

/// Laeuft irgendwo etwas - `pygame.mixer.get_busy()`.
///
/// Gemessen: das ist die **einzige** Funktion, die ohne `init()` keinen
/// Fehler wirft, sondern schlicht `false` liefert. Und: ein pausierter
/// Kanal zaehlt als belegt.
pub fn get_busy() -> bool {
    zustand::mit(|z| {
        z.offen.is_some() && (0..z.anzahl).any(|i| z.kanaele[i].lauf.is_some())
    })
}

/// Haelt alle Kanaele an - `pygame.mixer.stop()`.
///
/// Wie [`Channel::stop`]: Endereignisse fallen an, und Schlangen ruecken
/// auf.
pub fn stop() -> Ergebnis<()> {
    zustand::mit(|z| {
        z.pruefen()?;
        let jetzt = rustgame_core::time::get_ticks();
        for i in 0..z.anzahl {
            z.kanal_beenden(i, jetzt);
        }
        Ok(())
    })
}

/// Pausiert alle Kanaele - `pygame.mixer.pause()`.
///
/// Die Musik bleibt davon unberuehrt; die hat ihr eigenes
/// [`crate::music::pause`].
pub fn pause() -> Ergebnis<()> {
    zustand::mit(|z| {
        z.pruefen()?;
        let jetzt = rustgame_core::time::get_ticks();
        for i in 0..z.anzahl {
            let Some(l) = z.kanaele[i].lauf.as_mut() else {
                continue;
            };
            if l.pause_seit.is_none() {
                l.pause_seit = Some(jetzt);
                if let Some(t) = z.treiber.as_mut() {
                    t.pausieren(Stimme::Kanal(i));
                }
            }
        }
        Ok(())
    })
}

/// Laesst alle Kanaele weiterlaufen - `pygame.mixer.unpause()`.
pub fn unpause() -> Ergebnis<()> {
    zustand::mit(|z| {
        z.pruefen()?;
        let jetzt = rustgame_core::time::get_ticks();
        for i in 0..z.anzahl {
            let Some(l) = z.kanaele[i].lauf.as_mut() else {
                continue;
            };
            if let Some(seit) = l.pause_seit {
                l.pause_summe += jetzt.saturating_sub(seit);
                l.pause_seit = None;
                if let Some(t) = z.treiber.as_mut() {
                    t.fortsetzen(Stimme::Kanal(i));
                }
            }
        }
        Ok(())
    })
}

/// Blendet alle Kanaele aus - `pygame.mixer.fadeout(ms)`.
pub fn fadeout(millisekunden: i32) -> Ergebnis<()> {
    zustand::mit(|z| {
        z.pruefen()?;
        let jetzt = rustgame_core::time::get_ticks();
        for i in 0..z.anzahl {
            crate::channel::ausblenden_setzen(z, i, millisekunden, jetzt);
        }
        Ok(())
    })
}

// ---------------------------------------------------------------------------
//  Innereien
// ---------------------------------------------------------------------------

/// Das Ausgabeformat - oder [`Fehler::NichtInitialisiert`].
pub(crate) fn spec() -> Ergebnis<Spec> {
    zustand::mit(|z| z.pruefen())
}

/// Rechnet einen Lautstaerkewunsch in SDLs Stufen um.
///
/// Der Kern der 128-Stufen-Rasterung, und zugleich der Grund fuer eine
/// Eigenart, die sonst wie ein Fehler aussieht: ein **negativer** Wert
/// laesst die Lautstaerke stehen. In C landet er als negative Zahl in
/// `Mix_Volume`/`Mix_VolumeChunk`, und die lesen bei negativen Zahlen nur
/// ab, statt zu setzen. Gemessen: `set_volume(-0.5)` gefolgt von
/// `get_volume()` gibt `1.0` - den alten Wert, nicht `0.0`.
pub(crate) fn stufe(wert: f64, alt: i32) -> i32 {
    // Bewusst in einfacher Genauigkeit und gegen null abgeschnitten -
    // pygame rechnet `(int)(volume * 128)` mit einem C-`float`.
    let stufen = (wert as f32 * MAX_VOLUME as f32) as i32;
    if stufen < 0 {
        alt
    } else {
        stufen.min(MAX_VOLUME)
    }
}

/// Macht aus Wunsch und Voreinstellung das wirkliche Ausgabeformat.
fn auswerten(wunsch: Einstellungen, vorher: Einstellungen) -> Ergebnis<Spec> {
    let grund = |w: i32, v: i32, vorgabe: i32| {
        if w != 0 {
            w
        } else if v != 0 {
            v
        } else {
            vorgabe
        }
    };
    let frequenz = grund(wunsch.frequency, vorher.frequency, VORGABE_FREQUENZ);
    let groesse = grund(wunsch.size, vorher.size, VORGABE_GROESSE);
    let spuren = grund(wunsch.channels, vorher.channels, VORGABE_KANAELE);
    let puffer = grund(wunsch.buffer, vorher.buffer, VORGABE_PUFFER);

    // Gemessen: `size = 32` kommt als **-32** zurueck, weil daraus
    // `AUDIO_F32SYS` wird; `size = 8` bleibt `8`.
    let groesse = match groesse {
        8 | -8 | 16 | -16 => groesse,
        32 | -32 => -32,
        g => return Err(Fehler::Wert(format!("unsupported size {g}"))),
    };

    // Gemessen: 3 wird 2, 5 wird 4, alles darueber 6.
    let spuren = if spuren <= 1 {
        1
    } else if spuren <= 3 {
        2
    } else if spuren <= 5 {
        4
    } else {
        6
    };

    // Auf die naechste Zweierpotenz aufrunden, mindestens 256.
    let mut p = 1i32;
    while p < puffer && p < (1 << 30) {
        p <<= 1;
    }
    let puffer = p.max(256);

    if frequenz <= 0 {
        return Err(Fehler::Wert(format!("unsupported frequency {frequenz}")));
    }

    Ok(Spec {
        frequenz,
        groesse,
        spuren,
        puffer,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn leer() -> Einstellungen {
        Einstellungen::default()
    }

    #[test]
    fn vorgaben_stimmen_mit_pygame_ueberein() {
        // Gemessen: `mixer.init()` ohne Argumente ergibt (44100, -16, 2),
        // Puffer 512.
        let s = auswerten(leer(), leer()).unwrap();
        assert_eq!((s.frequenz, s.groesse, s.spuren), (44_100, -16, 2));
        assert_eq!(s.puffer, 512);
    }

    #[test]
    fn groesse_32_wird_negativ() {
        // Gemessen: init(size=32) -> get_init() == (44100, -32, 2).
        assert_eq!(
            auswerten(Einstellungen::neu(0, 32, 0, 0), leer())
                .unwrap()
                .groesse,
            -32
        );
        // Aber size=8 bleibt 8 - vorzeichenlos.
        assert_eq!(
            auswerten(Einstellungen::neu(0, 8, 0, 0), leer())
                .unwrap()
                .groesse,
            8
        );
    }

    #[test]
    fn spuren_werden_gerundet() {
        for (hinein, heraus) in [(1, 1), (2, 2), (3, 2), (4, 4), (5, 4), (6, 6), (8, 6)] {
            assert_eq!(
                auswerten(Einstellungen::neu(0, 0, hinein, 0), leer())
                    .unwrap()
                    .spuren,
                heraus,
                "channels={hinein}"
            );
        }
    }

    #[test]
    fn puffer_wird_auf_zweierpotenz_aufgerundet() {
        for (hinein, heraus) in [(1, 256), (256, 256), (300, 512), (1024, 1024), (1025, 2048)] {
            assert_eq!(
                auswerten(Einstellungen::neu(0, 0, 0, hinein), leer())
                    .unwrap()
                    .puffer,
                heraus,
                "buffer={hinein}"
            );
        }
    }

    #[test]
    fn stufen_rasten_wie_gemessen_ein() {
        // Alle Zahlen aus echtem pygame.
        assert_eq!(stufe(0.5, 128), 64);
        assert_eq!(stufe(0.3333333, 128), 42); // -> 0.328125
        assert_eq!(stufe(0.1, 128), 12); // -> 0.09375
        assert_eq!(stufe(0.01, 128), 1); // -> 0.0078125
        assert_eq!(stufe(1.0 / 255.0, 128), 0); // -> 0.0
        assert_eq!(stufe(1.5, 64), 128); // gekappt
        assert_eq!(stufe(-0.5, 128), 128); // aendert nichts
        assert_eq!(stufe(-0.5, 42), 42);
    }
}
