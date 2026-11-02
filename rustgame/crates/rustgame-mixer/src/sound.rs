//! sound.rs
//! ========
//! `pygame.mixer.Sound` - ein Klang im Speicher.
//!
//! ## Warum hier umgerechnet wird
//!
//! Ein `Sound` haelt seine Daten **nicht** so, wie sie in der Datei
//! standen, sondern im Ausgabeformat des Mixers. Das ist keine Bequem-
//! lichkeit, sondern gemessen: `Sound.get_raw()` einer Mono-Datei mit
//! 1000 Werten bei 44100 Hz liefert unter der Vorgabe `(44100, -16, 2)`
//! genau **4000** Bytes - vier je Wert, also Stereo mit 16 Bit. Und
//! dieselbe Datei mit 22050 Hz liefert **acht** Bytes je Wert, ist also
//! beim Laden hochgerechnet worden.
//!
//! Daran haengt auch [`Sound::get_length`]: die Laenge ist nicht die der
//! Datei, sondern `Rahmen / Mixerfrequenz`.
//!
//! ## Drei Wege hinein, und sie meinen Verschiedenes
//!
//! | hier | pygame | was drin steht |
//! |---|---|---|
//! | [`Sound::new`] | `Sound(file)` | ein Dateiname |
//! | [`Sound::from_bytes`] | `Sound(BytesIO(...))` | eine **Datei** als Bytes (WAV, OGG) |
//! | [`Sound::from_buffer`] | `Sound(buffer=...)` | **rohe Abtastwerte** im Mixerformat |
//!
//! Die letzten beiden werden gern verwechselt, und pygame hilft dabei
//! nicht: `Sound(buffer=offene_wav_datei.read())` wirft keinen Fehler,
//! sondern liest den WAV-**Kopf** als Klang mit. Gemessen kommt dabei
//! eine 0.0116 Sekunden lange Datei heraus statt 0.0227 - erkennbar nur
//! daran, dass die Zahl nicht stimmt. Deshalb heissen die beiden hier
//! verschieden.

use std::cell::Cell;
use std::rc::Rc;

use crate::channel::Channel;
use crate::konstanten::*;
use crate::mixer;
use crate::wav::{Klangdaten, Weg};
use crate::zustand::{self, Lauf};
use crate::{Ergebnis, Fehler};

/// Was ein [`Sound`] wirklich festhaelt.
#[derive(Debug)]
pub(crate) struct Klang {
    /// Die Abtastwerte, verschraenkt und **im Ausgabeformat des Mixers**.
    ///
    /// Hier stand `Vec<i16>` - immer 16-bittig, ganz gleich, womit der
    /// Mixer lief. Nach aussen stimmte das meiste: `get_raw()` rechnete
    /// beim Herausgeben um, `get_length()` zaehlte richtig. Nur die
    /// **Adresse** log: `Sound._samples_address` zeigte auf 16-Bit-Worte,
    /// waehrend `__array_interface__` unter `init(size=8)` `|u1`
    /// versprach. Wer dem Zeiger folgte, fand das Doppelte an Bytes.
    ///
    /// Seit `pygame.sndarray` da ist, folgt ihm jemand: `sndarray.array`
    /// und `sndarray.samples` bauen numpy-Felder genau darauf, und
    /// letzteres ist eine **lebende** Sicht - was hineingeschrieben wird,
    /// steht danach im Klang. Mit zwei Ablagen waere das nicht zu machen.
    ///
    /// Also liegt hier jetzt genau das, was SDL_mixer in `chunk->abuf`
    /// haelt. Gemischt wird weiter 16-bittig; [`Sound::werte`] rechnet
    /// beim Abspielen um, und das geschieht einmal je `play()` und nicht
    /// je Puffer.
    pub roh: Vec<u8>,
    /// Ausgabespuren des Mixers.
    pub spuren: u16,
    /// Ausgabefrequenz des Mixers.
    pub frequenz: u32,
    /// Bits je Wert des Ausgabeformats, wie `get_init()` sie meldet:
    /// `-16`, `16`, `-8`, `8` oder `-32`. Negativ heisst
    /// vorzeichenbehaftet, `-32` heisst Gleitkomma.
    ///
    /// **Das ist nicht die Breite von [`Klang::werte`].** Gemischt wird
    /// hier immer 16-bittig; dieses Feld sagt nur, in welchem Format der
    /// Klang **hereinkam und wieder herausgeht**. SDL_mixer legt den
    /// Klang wirklich im Ausgabeformat ab (`chunk->abuf`), und daran
    /// haengt eine gemessene Zahl: unter `init(size=-8)` liefert
    /// `get_raw()` fuer dieselbe Datei **halb so viele** Bytes wie unter
    /// `-16`, waehrend `get_length()` gleich bleibt. pygames eigener
    /// `test_get_length` rechnet genau diese Gleichung nach.
    ///
    /// Festgehalten wird das Format vom **Ladezeitpunkt**, so wie
    /// `chunk->abuf` es festhaelt: ein `init()` mit anderer Groesse
    /// laesst schon geladene Klaenge unveraendert.
    pub groesse: i32,
    /// Lautstaerke in SDLs Stufen, 0 bis 128.
    pub lautstaerke: Cell<i32>,
}

/// Ein Klang im Speicher - `pygame.mixer.Sound`.
///
/// Klonen ist billig und macht **denselben** Klang, nicht einen zweiten:
/// zwei Klone teilen Daten und Lautstaerke, so wie zwei Python-Namen auf
/// dasselbe `Sound`-Objekt zeigen. [`PartialEq`] vergleicht darum die
/// Herkunft, nicht den Inhalt - das ist pygames `is`.
#[derive(Debug, Clone)]
pub struct Sound {
    pub(crate) inner: Rc<Klang>,
}

impl PartialEq for Sound {
    fn eq(&self, other: &Sound) -> bool {
        Rc::ptr_eq(&self.inner, &other.inner)
    }
}

impl Eq for Sound {}

impl Sound {
    // -- Anlegen ----------------------------------------------------------

    /// Laedt eine Klangdatei - `pygame.mixer.Sound(dateiname)`.
    ///
    /// WAV geht immer. OGG geht ueber rodios `vorbis`-Merkmal. MP3 gibt
    /// einen sauberen [`Fehler::NichtUnterstuetzt`] - **keine Panik**.
    pub fn new(pfad: impl AsRef<std::path::Path>) -> Ergebnis<Sound> {
        Sound::from_bytes(&datei_lesen(pfad.as_ref())?)
    }

    /// Liest eine Klangdatei aus einem Byte-Puffer.
    ///
    /// Das Gegenstueck zu `pygame.mixer.Sound(io.BytesIO(daten))`, nicht
    /// zu `Sound(buffer=...)` - siehe [`Sound::from_buffer`].
    pub fn from_bytes(daten: &[u8]) -> Ergebnis<Sound> {
        let k = dekodieren(daten, Weg::Klang)?;
        Sound::from_klangdaten(k)
    }

    /// Liest ein Stueck so, wie `mixer.music` es liest.
    ///
    /// Derselbe Dekoder, aber **SDL_mixers** Erkennung und Saetze statt
    /// SDLs - siehe [`crate::wav::Weg`]. `music.load`, `music.queue` und
    /// `music.get_metadata` gehen hierueber.
    pub fn from_bytes_als_musik(daten: &[u8]) -> Ergebnis<Sound> {
        let k = dekodieren(daten, Weg::Musik)?;
        Sound::from_klangdaten(k)
    }

    /// Nimmt fertige Abtastwerte - `pygame.mixer.Sound(buffer=...)`.
    ///
    /// Die Bytes gelten als Werte **im Ausgabeformat des Mixers**, bereits
    /// in seiner Frequenz und Spurenzahl. Es wird nichts geprueft und
    /// nichts umgerechnet, genau wie bei pygame - nur ausgepackt.
    ///
    /// Welches Format das ist, sagt `get_init()[1]`: unter der Vorgabe
    /// `-16` sind es 16-Bit-Werte mit Vorzeichen, unter `8` einzelne
    /// vorzeichenlose Bytes. [`Sound::get_raw`] packt genau so wieder ein,
    /// der Weg hin und zurueck ist also verlustfrei.
    ///
    /// **Jedes Byte bleibt stehen**, auch ein halber Rahmen. Gemessen
    /// unter `init(44100, -16, 2)`, also vier Bytes je Rahmen:
    ///
    /// ```text
    /// Bytes hinein   4     5     6     7     8     9
    /// get_raw()      4     5     6     7     8     9
    /// get_length()  2,3e-5 ..............  4,5e-5 4,5e-5
    /// ```
    ///
    /// Die Laenge zaehlt also **ganze** Rahmen, der Puffer haelt alles.
    /// Hier wurde einmal beim Ablegen abgeschnitten - das gab dieselbe
    /// Laenge und einen kuerzeren `get_raw()`.
    pub fn from_buffer(daten: &[u8]) -> Ergebnis<Sound> {
        let spec = mixer::spec()?;
        Ok(Sound {
            inner: Rc::new(Klang {
                roh: daten.to_vec(),
                spuren: spec.spuren as u16,
                frequenz: spec.frequenz as u32,
                groesse: spec.groesse,
                lautstaerke: Cell::new(MAX_VOLUME),
            }),
        })
    }

    /// Nimmt fertige 16-Bit-Werte **im Format des Mixers**.
    ///
    /// Der Weg fuer `pygame.mixer.Sound(array=...)`. Anders als
    /// [`Sound::from_samples`] wird hier **nicht** umgerechnet: die Werte
    /// gelten als schon in der Frequenz und Spurenzahl des Mixers, so wie
    /// pygame ein Feld auch nimmt.
    ///
    /// Die Spurenzahl kommt vom Mixer und nicht vom Aufrufer - pygame
    /// prueft die Form des Feldes dagegen und sagt
    /// `Array must be 2-dimensional for stereo mixer`, wenn sie nicht
    /// passt. Die Pruefung selbst steht in der Python-Huelle, weil nur
    /// dort die Form des Feldes bekannt ist.
    pub fn from_array(werte: &[i16]) -> Ergebnis<Sound> {
        let spec = mixer::spec()?;
        Ok(Sound {
            inner: Rc::new(Klang {
                roh: roh_schreiben(werte, spec.groesse),
                spuren: spec.spuren as u16,
                frequenz: spec.frequenz as u32,
                groesse: spec.groesse,
                lautstaerke: Cell::new(MAX_VOLUME),
            }),
        })
    }

    /// Wie viele Spuren der Mixer gerade hat - fuer `Sound(array=...)`.
    pub fn mixer_spuren() -> Ergebnis<u16> {
        Ok(mixer::spec()?.spuren as u16)
    }

    /// Das Ausgabeformat des Mixers, wie `get_init()[1]` es meldet.
    pub fn mixer_groesse() -> Ergebnis<i32> {
        Ok(mixer::spec()?.groesse)
    }

    /// Nimmt Abtastwerte und rechnet sie auf das Mixerformat um.
    ///
    /// Der bequeme Weg fuer selbst erzeugte Klaenge - siehe
    /// [`crate::synth`].
    pub fn from_samples(werte: &[i16], frequenz: u32, spuren: u16) -> Ergebnis<Sound> {
        Sound::from_klangdaten(Klangdaten {
            werte: werte.to_vec(),
            spuren,
            frequenz,
        })
    }

    /// Ein zweiter, eigenstaendiger Klang mit denselben Daten -
    /// `Sound.copy()`.
    ///
    /// Anders als [`Clone`], das absichtlich **denselben** Klang
    /// zurueckgibt (zwei Python-Namen auf ein Objekt), legt das hier eine
    /// eigene Kopie an: die Lautstaerke der Kopie laesst sich verstellen,
    /// ohne dass das Original davon etwas merkt.
    ///
    /// Uebertragen aus `src_c/mixer.c`, `snd_copy`: pygame ruft dort
    /// `snd_get_raw` und gibt die Bytes an `Sound(buffer=...)` weiter -
    /// der Umweg ist moeglich, weil ein Klang seine Daten ohnehin im
    /// Ausgabeformat des Mixers haelt. Zum Schluss traegt es die
    /// Lautstaerke des Originals nach
    /// (`Mix_VolumeChunk(dst, Mix_VolumeChunk(orig, -1))`); gemessen ist
    /// `s.set_volume(0.3); s.copy().get_volume()` genau `0.296875`.
    ///
    /// `MIXER_INIT_CHECK()` steht dort **vor** allem anderen - gemessen
    /// gibt `s.copy()` nach `mixer.quit()` `mixer not initialized`.
    ///
    /// Der Umweg ueber die Bytes wird hier **wirklich gegangen** und
    /// nicht abgekuerzt. Er ist naemlich sichtbar, sobald sich das
    /// Ausgabeformat zwischen Laden und Kopieren geaendert hat: die Bytes
    /// des Originals stehen dann im alten Format da und werden im neuen
    /// gelesen. Das ist keine Schoenheit, sondern genau das, was pygame
    /// tut - `chunk->abuf` traegt sein Ladeformat mit sich.
    pub fn copy(&self) -> Ergebnis<Sound> {
        let neu = Sound::from_buffer(&self.get_raw())?;
        neu.inner.lautstaerke.set(self.inner.lautstaerke.get());
        Ok(neu)
    }

    fn from_klangdaten(k: Klangdaten) -> Ergebnis<Sound> {
        let spec = mixer::spec()?;
        let werte = umrechnen(&k, spec.frequenz as u32, spec.spuren as u16);
        Ok(Sound {
            inner: Rc::new(Klang {
                roh: roh_schreiben(&werte, spec.groesse),
                spuren: spec.spuren as u16,
                frequenz: spec.frequenz as u32,
                groesse: spec.groesse,
                lautstaerke: Cell::new(MAX_VOLUME),
            }),
        })
    }

    // -- Abfragen ---------------------------------------------------------

    /// Die Laenge in Sekunden - `Sound.get_length()`.
    ///
    /// **Gemessen und genau so nachgebaut:** pygame rechnet das in
    /// einfacher Genauigkeit (`(float)numsamples / (float)freq`) und
    /// weitet das Ergebnis erst danach auf. Eine Datei mit 1000 Werten
    /// bei 44100 Hz liefert darum `0.022675737738609314` - und nicht die
    /// mathematisch richtigen `0.022675736961451247`. Wer gegen pygame
    /// prueft, stolpert genau hier.
    pub fn get_length(&self) -> f64 {
        if self.inner.frequenz == 0 {
            return 0.0;
        }
        (self.rahmen() as f32 / self.inner.frequenz as f32) as f64
    }

    /// Die Werte als Bytes - `Sound.get_raw()`.
    ///
    /// Verschraenkt, in der Frequenz und Spurenzahl des **Mixers** -
    /// nicht in der der Datei - und im Ausgabeformat, das beim Laden galt
    /// (siehe `Klang::groesse`).
    ///
    /// Gemessen an `punch.wav` (16164 Rahmen, zwei Spuren):
    ///
    /// ```text
    /// init(size=-16) -> 64656 Bytes    init(size=-8) -> 32328 Bytes
    /// init(size= 16) -> 64656 Bytes    init(size= 8) -> 32328 Bytes
    /// ```
    ///
    /// und `get_length()` bleibt bei allen vier `0.36653063`. Wer das
    /// uebersieht, baut einen Nachbau, dessen `get_raw()` immer 16-bittig
    /// ist - und faellt ueber pygames `test_get_length`, das
    /// `len(get_raw()) / freq / (abs(bits)/8) / spuren` gegen
    /// `get_length()` haelt und dann genau den Faktor 2 danebenliegt.
    pub fn get_raw(&self) -> Vec<u8> {
        self.inner.roh.clone()
    }

    /// Die Lautstaerke von 0.0 bis 1.0 - `Sound.get_volume()`.
    pub fn get_volume(&self) -> f64 {
        self.inner.lautstaerke.get() as f64 / MAX_VOLUME as f64
    }

    /// Stellt die Lautstaerke ein - `Sound.set_volume()`.
    ///
    /// Gemessen: der Wert rastet auf 128 Stufen ein, ueber 1.0 wird
    /// gekappt, und ein **negativer** Wert aendert gar nichts - SDLs
    /// `Mix_VolumeChunk` liest bei negativen Zahlen nur ab, statt zu
    /// setzen. `set_volume(-0.5)` laesst die Lautstaerke also stehen, wo
    /// sie war.
    pub fn set_volume(&self, wert: f64) {
        let alt = self.inner.lautstaerke.get();
        self.inner.lautstaerke.set(mixer::stufe(wert, alt));
    }

    /// Auf wie vielen Kanaelen dieser Klang gerade laeuft -
    /// `Sound.get_num_channels()`.
    pub fn get_num_channels(&self) -> usize {
        zustand::mit(|z| {
            z.kanaele[..z.anzahl]
                .iter()
                .filter(|k| k.lauf.is_some() && k.sound.as_ref() == Some(self))
                .count()
        })
    }

    // -- Spielen ----------------------------------------------------------

    /// Spielt den Klang auf dem naechsten freien Kanal -
    /// `Sound.play(loops, maxtime, fade_ms)`.
    ///
    /// `loops` zaehlt die **Wiederholungen**, nicht die Durchlaeufe:
    /// gemessen spielt `loops=1` zweimal und `loops=0` einmal; `-1` ist
    /// endlos. (Die Musikspur zaehlt anders, siehe
    /// [`crate::music::play`].)
    ///
    /// `maxtime` bricht nach so vielen Millisekunden ab; `0` und
    /// negative Werte heissen "keine Schranke" - gemessen laeuft
    /// `maxtime=0` die volle Laenge.
    ///
    /// Gibt `None`, wenn **alle** Kanaele belegt sind. Gemessen: pygame
    /// wirft dort keinen Fehler, sondern liefert `None` - und
    /// `get_num_channels()` bleibt stehen.
    ///
    /// Anders als [`Channel::play`] setzt das hier die Lautstaerke des
    /// benutzten Kanals auf voll zurueck (`Mix_Volume(ch, 128)`) -
    /// gemessen, und ein haeufiger Stolperstein, wenn man den Kanal
    /// vorher leiser gestellt hat.
    pub fn play(&self, loops: i32, maxtime: i32, fade_ms: i32) -> Option<Channel> {
        zustand::mit(|z| {
            // Ohne laufenden Mixer gibt es keinen Kanal - und gemessen
            // auch keinen Fehler, sondern None.
            z.offen?;
            let i = freier_kanal(z)?;
            z.kanaele[i].lautstaerke = MAX_VOLUME;
            anwerfen(z, i, self, loops, maxtime, fade_ms);
            Some(Channel::roh(i))
        })
    }

    /// Haelt den Klang auf allen Kanaelen an - `Sound.stop()`.
    ///
    /// Achtung, gemessen: wo eine Warteschlange haengt, rueckt sie dabei
    /// auf - siehe [`Channel::stop`].
    pub fn stop(&self) {
        zustand::mit(|z| {
            let jetzt = rustgame_core::time::get_ticks();
            for i in 0..z.anzahl {
                if z.kanaele[i].sound.as_ref() == Some(self) {
                    z.kanal_beenden(i, jetzt);
                }
            }
        });
    }

    /// Blendet den Klang auf allen Kanaelen aus - `Sound.fadeout(ms)`.
    pub fn fadeout(&self, millisekunden: i32) {
        zustand::mit(|z| {
            let jetzt = rustgame_core::time::get_ticks();
            for i in 0..z.anzahl {
                if z.kanaele[i].sound.as_ref() == Some(self) {
                    crate::channel::ausblenden_setzen(z, i, millisekunden, jetzt);
                }
            }
        });
    }

    // -- fuer die Kiste ---------------------------------------------------

    /// Die Werte als 16-Bit-Zahlen - **gerechnet, nicht gelagert**.
    ///
    /// Der Mixer rechnet in 16 Bit, der Klang liegt im Ausgabeformat. Die
    /// Umrechnung geschieht hier, und sie geschieht einmal je `play()`:
    /// der Treiber kopiert die Werte ohnehin in seinen eigenen Puffer
    /// (`auftrag.werte.to_vec()`), es kommt also keine Kopie dazu.
    pub(crate) fn werte(&self) -> Vec<i16> {
        roh_lesen(&self.inner.roh, self.inner.groesse)
    }

    /// Wie viele Spuren verschraenkt in [`Sound::get_raw`] stehen.
    pub fn spuren(&self) -> u16 {
        self.inner.spuren
    }

    pub(crate) fn frequenz(&self) -> u32 {
        self.inner.frequenz
    }

    /// Das Ausgabeformat, in dem dieser Klang abgelegt ist - `-16`, `16`,
    /// `-8`, `8` oder `-32`, wie `get_init()[1]`.
    ///
    /// Gebraucht wird das von der Python-Huelle fuer
    /// `Sound.__array_interface__`: dort steht das Format als `typestr`
    /// (`<i2`, `|u1`, ...), und es muss zu dem passen, was
    /// [`Sound::get_raw`] liefert.
    pub fn groesse(&self) -> i32 {
        self.inner.groesse
    }

    /// Die Stelle der Abtastwerte im Speicher - `Sound._samples_address`.
    ///
    /// In pygame ist das `chunk->abuf`, also der Anfang des Puffers, den
    /// auch `get_raw()` ausliest. Hier ist es der Anfang von
    /// `Klang::werte`, und das ist **nicht** ganz dasselbe: gemischt
    /// wird hier immer 16-bittig, unter `init(size=8)` liegen an dieser
    /// Stelle also 16-Bit-Werte, waehrend pygame dort Bytes haette. Im
    /// Regelfall (`-16`, die Vorgabe) stimmen beide ueberein.
    ///
    /// **Kein `unsafe`:** `as_ptr` ist ein sicherer Aufruf; gelesen wird
    /// ueber diesen Zeiger hier nirgends. Er wandert als blosse Zahl nach
    /// Python, so wie in pygame auch - wer ihn dort verfolgt, tut das auf
    /// eigene Rechnung.
    ///
    /// Die Zahl bleibt gleich, solange der Klang lebt: `werte` wird nach
    /// dem Anlegen nicht mehr veraendert, und das `Rc` haelt ihn fest.
    pub fn zeiger(&self) -> usize {
        self.inner.roh.as_ptr() as usize
    }

    /// Eine Zahl, die genau diesen Klang bezeichnet.
    ///
    /// Zwei Klone geben dieselbe, zwei getrennt geladene Klaenge
    /// verschiedene - es ist die Stelle der gemeinsamen Daten im
    /// Speicher, also genau das, was [`PartialEq`] hier vergleicht.
    ///
    /// Gebraucht wird das von der Python-Huelle: dort muss ein `Sound`
    /// hashbar sein (in pygame ist er es, gemessen), und `__hash__` muss
    /// zu `__eq__` passen. Ohne diesen Zugang bliebe nur, die Werte zu
    /// hashen - bei einem Klang von einer Sekunde waeren das 176400
    /// Bytes bei jedem Woerterbuchzugriff.
    ///
    /// **Kein Zeiger, sondern eine Zahl**, und sie wird nach dem Ende des
    /// Klangs wieder vergeben. Zum Wiederfinden taugt sie nicht, nur zum
    /// Unterscheiden.
    pub fn kennung(&self) -> usize {
        Rc::as_ptr(&self.inner) as *const () as usize
    }

    /// Wie viele Rahmen der Klang hat - Werte geteilt durch Spuren.
    pub fn rahmen(&self) -> usize {
        if self.inner.spuren == 0 {
            0
        } else {
            self.inner.roh.len()
                / (self.inner.spuren as usize * bytes_je_wert(self.inner.groesse).max(1))
        }
    }
}

// ---------------------------------------------------------------------------
//  Gemeinsames fuer Sound::play und Channel::play
// ---------------------------------------------------------------------------

/// Der naechste Kanal, der frei ist - oder `None`.
///
/// Gemessen: `Sound.play()` faengt bei den **reservierten** Kanaelen an zu
/// zaehlen und laesst sie aus. Nach `set_reserved(2)` landet der erste
/// Klang auf Kanal 2, nicht auf Kanal 0. (`find_channel()` macht das
/// **nicht** - das ist kein Tippfehler, sondern gemessen: es liefert dort
/// weiter Kanal 0.)
pub(crate) fn freier_kanal(z: &zustand::Zustand) -> Option<usize> {
    (z.reserviert..z.anzahl).find(|&i| z.kanaele[i].lauf.is_none())
}

/// Legt einen Lauf an und schickt ihn an den Treiber.
///
/// Die Kanallautstaerke wird hier **nicht** angefasst. Das ist der
/// Unterschied zwischen den beiden Wegen: `Channel.play` laesst sie
/// stehen, `Sound.play` setzt sie vorher selbst auf voll - gemessen.
pub(crate) fn anwerfen(
    z: &mut zustand::Zustand,
    i: usize,
    sound: &Sound,
    loops: i32,
    maxtime: i32,
    fade_ms: i32,
) {
    let jetzt = rustgame_core::time::get_ticks();
    let durchlaeufe = if loops < 0 { -1 } else { loops + 1 };
    let maxtime = if maxtime > 0 {
        Some(maxtime as u64)
    } else {
        None
    };
    z.kanaele[i].sound = Some(sound.clone());
    z.kanaele[i].schlange = None;
    z.kanaele[i].lauf = Some(Lauf {
        start: jetzt,
        pause_seit: None,
        pause_summe: 0,
        rahmen: sound.rahmen() as u64,
        frequenz: sound.frequenz(),
        durchlaeufe,
        ab_ms: 0,
        maxtime,
        ausblenden: None,
        pos_versatz: 0,
    });
    z.beauftragen(i, 0, fade_ms.max(0) as u64);
}

// ---------------------------------------------------------------------------
//  Das Ausgabeformat ein- und auspacken
// ---------------------------------------------------------------------------

/// Wie viele Bytes ein Wert im Format `groesse` braucht.
///
/// Gemessen an pygames `_format_itemsize`: `8`/`-8` ein Byte, `16`/`-16`
/// zwei, `32`/`-32` vier.
pub fn bytes_je_wert(groesse: i32) -> usize {
    (groesse.unsigned_abs() as usize) / 8
}

/// Packt die 16-Bit-Werte in das Ausgabeformat - fuer
/// [`Sound::get_raw`].
///
/// Die Umrechnungen sind die von SDLs `SDL_ConvertAudio`: nach 8 Bit wird
/// um acht Stellen geschoben, vorzeichenlos zusaetzlich um 128 versetzt;
/// nach Gleitkomma wird durch 32768 geteilt. Der Weg mit
/// [`roh_lesen`] zurueck ist verlustfrei, bei jedem der fuenf Formate -
/// darauf baut `Sound.copy()`.
fn roh_schreiben(werte: &[i16], groesse: i32) -> Vec<u8> {
    let mut heraus = Vec::with_capacity(werte.len() * bytes_je_wert(groesse).max(1));
    for &w in werte {
        match groesse {
            8 => heraus.push(((w >> 8) as i32 + 128) as u8),
            -8 => heraus.push((w >> 8) as i8 as u8),
            16 => heraus.extend_from_slice(&((w as i32 + 32_768) as u16).to_le_bytes()),
            -32 => heraus.extend_from_slice(&(w as f32 / 32_768.0).to_le_bytes()),
            // `-16` und alles Unbekannte: das Mischformat selbst.
            _ => heraus.extend_from_slice(&w.to_le_bytes()),
        }
    }
    heraus
}

/// Das Gegenstueck zu [`roh_schreiben`] - fuer [`Sound::from_buffer`].
///
/// Was nicht in einen ganzen Wert passt, faellt hinten weg.
fn roh_lesen(daten: &[u8], groesse: i32) -> Vec<i16> {
    match groesse {
        8 => daten.iter().map(|&b| ((b as i32 - 128) << 8) as i16).collect(),
        -8 => daten.iter().map(|&b| ((b as i8 as i32) << 8) as i16).collect(),
        16 => daten
            .as_chunks::<2>().0.iter()
            .map(|c| (u16::from_le_bytes([c[0], c[1]]) as i32 - 32_768) as i16)
            .collect(),
        -32 => daten
            .as_chunks::<4>().0.iter()
            .map(|c| {
                (f32::from_le_bytes([c[0], c[1], c[2], c[3]]) * 32_768.0)
                    .clamp(-32_768.0, 32_767.0) as i16
            })
            .collect(),
        _ => daten
            .as_chunks::<2>().0.iter()
            .map(|c| i16::from_le_bytes([c[0], c[1]]))
            .collect(),
    }
}

// ---------------------------------------------------------------------------
//  Dekodieren und Umrechnen
// ---------------------------------------------------------------------------

/// Liest eine Klangdatei ein - mit der Fehlermeldung, die pygame gibt.
///
/// Steht hier und nicht in [`Sound::new`], weil `music::load` dieselben
/// Rohbytes braucht: die ID3-Etiketten stehen darin und nicht in den
/// Abtastwerten. Zweimal lesen waere die Alternative gewesen.
pub(crate) fn datei_lesen(pfad: &std::path::Path) -> Ergebnis<Vec<u8>> {
    // **`e.kind()`, nicht `e`.** Die Anzeige eines `io::Error` ist der
    // Satz des Betriebssystems in **dessen** Sprache - hier stand
    // `No file 'rustgame' found: Zugriff verweigert (os error 5)`. Die Art
    // des Fehlers beschreibt Rust selbst, und zwar englisch.
    std::fs::read(pfad).map_err(|e| {
        Fehler::Datei(format!("No file '{}' found: {}", pfad.display(), e.kind()))
    })
}

/// Sagt nur, **ob** es ein Tonformat ist - ohne es zu lesen.
///
/// `music.get_metadata` braucht genau das: pygame gibt dort fuer eine
/// Datei, die kein Tonformat ist, einen Fehler statt leerer Etiketten,
/// liest sie aber nicht ein. Siehe [`crate::music::metadaten_von`].
pub(crate) fn dekodierer_pruefen(daten: &[u8]) -> Ergebnis<()> {
    if daten.len() < 12 {
        return Err(Fehler::Format(
            "Couldn't read first 12 bytes of audio data".into(),
        ));
    }
    let bekannt = matches!(&daten[0..4], b"RIFF" | b"FORM" | b"OggS" | b"fLaC" | b"wvpk")
        || daten.starts_with(b"ID3")
        || (daten[0] == 0xFF && (daten[1] & 0xE0) == 0xE0)
        || daten.starts_with(b"Extended Module:");
    if bekannt {
        Ok(())
    } else {
        Err(Fehler::Format("Unrecognized audio format".into()))
    }
}

/// Erkennt am Anfang der Daten, was es ist, und liest es.
///
/// ## Vier Bytes oder zwoelf
///
/// `Mix_LoadWAV_RW` (der Weg von `Sound`) liest erst **vier** Bytes und
/// reicht `RIFF` und `WAVE` sofort an `SDL_LoadWAV_RW` weiter; alles andere
/// geht an die Musikleser, und die wollen zwoelf. `Mix_LoadMUS_RW` (der
/// Weg von `music`) liest gleich zwoelf. Gemessen:
///
/// ```text
///                 Sound                                      music
/// b""             Couldn't read first 4 bytes of audio data  .. 12 bytes ..
/// b"RIFF"         Could not read RIFF header                 .. 12 bytes ..
/// b"RIFF....WAV"  Could not read RIFF form type              .. 12 bytes ..
/// b"OggS"         Couldn't read first 12 bytes of audio data .. 12 bytes ..
/// ```
fn dekodieren(daten: &[u8], weg: Weg) -> Ergebnis<Klangdaten> {
    if weg == Weg::Klang {
        if daten.len() < 4 {
            return Err(Fehler::Format(
                "Couldn't read first 4 bytes of audio data".into(),
            ));
        }
        if matches!(&daten[0..4], b"RIFF" | b"WAVE") {
            return crate::wav::lesen_als(daten, weg);
        }
        // `Mix_LoadWAV_RW`: nach `RIFF`/`WAVE` kommt `FORM` - AIFF und
        // 8SVX, mit SDL_mixers eigenen Saetzen (`load_aiff.c`).
        if &daten[0..4] == b"FORM" {
            return crate::aiff::klang_lesen(daten);
        }
        // ... und `Crea` - Creative Voice (`load_voc.c`). Nur hier; fuer
        // `music` kennt SDL_mixer kein VOC.
        if &daten[0..4] == b"Crea" {
            return crate::voc::klang_lesen(daten);
        }
    }
    if daten.len() < 12 {
        return Err(Fehler::Format(
            "Couldn't read first 12 bytes of audio data".into(),
        ));
    }
    let ergebnis = match &daten[0..4] {
        // Fuer `music` ist nur `RIFF....WAVE` eine WAV-Datei; ein anderer
        // RIFF-Formtyp faellt durch bis zum Ende.
        b"RIFF" if &daten[8..12] == b"WAVE" => crate::wav::lesen_als(daten, weg),
        // `detect_music_type`: `FORM` ist fuer `music` eine WAV-Datei, und
        // `music_wav.c` liest daraus AIFF und AIFF-C.
        b"FORM" => crate::aiff::musik_lesen(daten),
        // **Zwei Sachen in einem Behaelter.** `OggS` sagt nur, dass es
        // ein Ogg ist; darin kann Vorbis stecken oder Opus. Was es ist,
        // steht im ersten Paket: `OpusHead` oder `\x01vorbis`.
        b"OggS" if opus_verdacht(daten) => opus_lesen(daten),
        b"OggS" => ogg_lesen(daten),
        b"fLaC" => flac_lesen(daten),
        b"ID3 " | b"ID3\x03" | b"ID3\x04" => mp3_lesen(daten),
        _ if daten[0] == b'I' && daten[1] == b'D' && daten[2] == b'3' => mp3_lesen(daten),
        // MPEG-Rahmenkopf: elf Einsen am Stueck.
        _ if daten[0] == 0xFF && (daten[1] & 0xE0) == 0xE0 => mp3_lesen(daten),
        // `wvpk` ist WavPack, `Extended Module:` ein XM-Stueck. Beide
        // nennt pygames Testsuite ausdruecklich.
        b"wvpk" => wavpack_lesen(daten),
        _ if daten.starts_with(b"Extended Module:") => xm_lesen(daten),
        // **Die anderen Trackerformate** - bei pygame liest sie libxmp,
        // hier xmrs (siehe `modul_lesen`).
        b"IMPM" => modul_lesen(daten, Modulart::It),
        // libxmps `s3m_test` will `SCRM` bei 44 **und** Dateityp 0x10 bei
        // 29 - nur die Kennung allein ist fuer libxmp kein S3M.
        _ if daten.get(44..48) == Some(b"SCRM") && daten.get(29) == Some(&0x10) => {
            modul_lesen(daten, Modulart::S3m)
        }
        _ if ist_mod(daten) => modul_lesen(daten, Modulart::Mod),
        // **Auf dem Weg von `music` hat das Unbekannte libxmps Namen.**
        // `detect_music_type` haelt alles ohne Kennung fuer ein Modul
        // ("there is no way to check if the file is really a MOD"), und
        // der letzte Leser, der es versucht, ist libxmp - gemessen an
        // Text, Nullen, GIF und `RIFF....AVI `. Nur `MThd` geht an die
        // MIDI-Leser und bleibt bei SDL_mixers eigenem Satz.
        _ if weg == Weg::Musik && !daten.starts_with(b"MThd") => Err(Fehler::Format(
            "XMP: Unrecognized file format".into(),
        )),
        _ => Err(Fehler::Format(
            "Unrecognized audio format".into(),
        )),
    };
    // **Auf dem Weg von `Sound` hat ein kaputter Klang keinen Namen.**
    // `Mix_LoadWAV_RW` reicht alles, was nicht WAV oder AIFF ist, still an
    // die Musikleser und sagt danach nur `Unrecognized audio format` -
    // gemessen an kaputten WavPack-, XM-, OGG-, Opus-, FLAC- und
    // MP3-Daten, jedes Mal derselbe Satz. Auf dem Weg von `music` meldet
    // sich dort der Dekodierer selbst (`stb_vorbis_open_rwops:
    // VORBIS_invalid_setup`, `music_drflac: corrupt flac file (bad
    // stream).`, auch `Out of memory`); diese Saetze haengen an den
    // Innereien von stb_vorbis, dr_flac und minimp3, und hier stehen
    // andere Dekodierer. Dort bleibt rustgames eigener Satz - eine
    // benannte Grenze, siehe `GRENZEN.md`.
    match weg {
        Weg::Klang => ergebnis.map_err(|f| match f {
            Fehler::Format(_) | Fehler::NichtUnterstuetzt(_) => {
                Fehler::Format("Unrecognized audio format".into())
            }
            andere => andere,
        }),
        Weg::Musik => ergebnis,
    }
}

/// Liest OGG/Vorbis ueber rodios Dekoder.
fn ogg_lesen(daten: &[u8]) -> Ergebnis<Klangdaten> {
    ueber_rodio(daten, "OGG", rodio::Decoder::new_vorbis)
}

/// Liest FLAC ueber rodios Dekoder (claxon, reines Rust).
fn flac_lesen(daten: &[u8]) -> Ergebnis<Klangdaten> {
    ueber_rodio(daten, "FLAC", rodio::Decoder::new_flac)
}

/// Liest MP3 ueber rodios Dekoder (symphonia, reines Rust).
///
/// **Der ID3-Block wird vorher abgeschnitten**, und das ist kein Vorgriff:
/// symphonia liest ihn selbst und scheitert an
/// `examples/data/metadata.mp3`, weil dort eine falsche Blocklaenge steht
/// (`out of bounds`). Warum genau, steht in [`crate::id3`].
fn mp3_lesen(daten: &[u8]) -> Ergebnis<Klangdaten> {
    let ab = crate::id3::tonbeginn_von(daten);
    ueber_rodio(&daten[ab..], "MP3", rodio::Decoder::new_mp3)
}

/// Der gemeinsame Rumpf der drei: rodios Dekoder anwerfen und die Werte
/// einsammeln.
///
/// Es gibt ihn, weil die drei sich nur im Namen und im Einsprungpunkt
/// unterschieden - und weil die Meldung das Format nennen soll, statt
/// rodios Wortlaut allein zu zeigen.
fn ueber_rodio<F>(daten: &[u8], name: &str, bauen: F) -> Ergebnis<Klangdaten>
where
    F: FnOnce(
        std::io::Cursor<Vec<u8>>,
    ) -> Result<rodio::Decoder<std::io::Cursor<Vec<u8>>>, rodio::decoder::DecoderError>,
{
    use rodio::Source;
    let leser = std::io::Cursor::new(daten.to_vec());
    let dek = bauen(leser).map_err(|e| Fehler::Format(format!("{name} could not be read: {e}")))?;
    let spuren = dek.channels();
    let frequenz = dek.sample_rate();
    let werte: Vec<i16> = dek.collect();
    Ok(Klangdaten {
        werte,
        spuren,
        frequenz,
    })
}

/// Rechnet Abtastrate und Spurenzahl auf das Mixerformat um.
///
/// **Das macht seit dem 29.09.2026 SDLs Filter**, nicht mehr eine lineare
/// Interpolation - siehe [`crate::umrechnen`]. Hier stand:
///
/// ```text
/// Lineare Interpolation zwischen den zwei naechsten Rahmen.
/// ```
///
/// Das war nicht falsch, nur anders: die Rahmenzahl stimmte exakt mit
/// pygame ueberein, die Werte nicht. Aufgefallen ist es erst, als bei
/// ADPCM die Byte-Zahl passte und der SHA-256 nicht.
///
/// Die Rahmenzahl ist weiterhin **abgeschnitten**, nicht gerundet -
/// gemessen: 1000 Werte bei 48000 Hz ergeben unter 44100 Hz genau 918
/// Rahmen, nicht 919 (mathematisch waeren es 918.75). SDL rechnet
/// `(Sint64)inframes * outrate / inrate`, und ganzzahlige Division
/// schneidet ab.
pub(crate) fn umrechnen(k: &Klangdaten, ziel_frequenz: u32, ziel_spuren: u16) -> Vec<i16> {
    crate::umrechnen::umrechnen(
        &k.werte,
        k.spuren,
        k.frequenz,
        ziel_spuren,
        ziel_frequenz,
    )
}

// ---------------------------------------------------------------------------
//  WavPack
// ---------------------------------------------------------------------------

/// Liest WavPack - verlustfrei, in reinem Rust.
///
/// **Ein eigener Behaelter, nicht nur ein Kodierer.** Eine `.wv`-Datei
/// besteht aus Bloecken mit der Kennung `wvpk`; jeder traegt seinen
/// eigenen Kopf mit Abtastrate, Spurzahl und Tiefe. `symphonia-codec-wavpack`
/// bringt beides mit - einen Leser fuer die Bloecke und den Dekodierer -,
/// und es ist ausdruecklich `#![forbid(unsafe_code)]`.
///
/// Gemessen an `examples/data/house_lo.wv`: 44100 Hz, zwei Spuren.
///
/// Der Dekodierer gibt Werte in der Tiefe der Datei; hier wird alles auf
/// 16 Bit gebracht, weil das die Form ist, in der rustgames Mischer
/// rechnet.
fn wavpack_lesen(daten: &[u8]) -> Ergebnis<Klangdaten> {
    
    use symphonia_core::codecs::audio::{AudioDecoder, AudioDecoderOptions};
    use symphonia_core::formats::{FormatOptions, FormatReader};
    use symphonia_core::io::{MediaSourceStream, MediaSourceStreamOptions};

    let quelle = std::io::Cursor::new(daten.to_vec());
    let strom = MediaSourceStream::new(Box::new(quelle), MediaSourceStreamOptions::default());
    let mut leser = symphonia_codec_wavpack::WavPackReader::try_new(strom, FormatOptions::default())
        .map_err(|e| Fehler::Format(format!("WavPack could not be read: {e}")))?;

    let spur = leser
        .tracks()
        .first()
        .ok_or_else(|| Fehler::Format("WavPack could not be read: no track".into()))?;
    let symphonia_core::codecs::CodecParameters::Audio(masse) = spur
        .codec_params
        .clone()
        .ok_or_else(|| Fehler::Format("WavPack could not be read: no codec parameters".into()))?
    else {
        return Err(Fehler::Format("WavPack could not be read: not an audio track".into()));
    };
    let frequenz = masse.sample_rate.unwrap_or(44100);
    let spuren = masse.channels.as_ref().map(|c| c.count()).unwrap_or(2) as u16;

    let mut dek = symphonia_codec_wavpack::WavPackDecoder::try_new(
        &masse,
        &AudioDecoderOptions::default(),
    )
    .map_err(|e| Fehler::Format(format!("WavPack could not be read: {e}")))?;

    let mut werte: Vec<i16> = Vec::new();
    while let Ok(Some(paket)) = leser.next_packet() {
        let Ok(puffer) = dek.decode_ref(&(&paket).into()) else {
            continue;
        };
        // **Verschachtelt, nicht spurweise.** symphonia legt die Spuren
        // getrennt ab (erst alle linken Werte, dann alle rechten);
        // rustgames Mischer erwartet sie abwechselnd. `copy_to_slice_interleaved`
        // macht genau das - und rechnet nebenbei auf 16 Bit herunter,
        // egal in welcher Tiefe die Datei liegt.
        let rahmen = puffer.frames();
        let spurzahl = puffer.spec().channels().count();
        let anfang = werte.len();
        werte.resize(anfang + rahmen * spurzahl, 0);
        puffer.copy_to_slice_interleaved::<i16, _>(&mut werte[anfang..]);
    }
    Ok(Klangdaten {
        werte,
        spuren,
        frequenz,
    })
}

// ---------------------------------------------------------------------------
//  XM (Fast Tracker II)
// ---------------------------------------------------------------------------

/// Spielt ein XM-Stueck ab und faengt die Werte auf.
///
/// **Ein Modul ist kein Tonstrom, sondern ein Notenblatt.** In der Datei
/// stehen Instrumente (kurze Klangschnipsel) und ein Muster, das sagt,
/// wann welches in welcher Tonhoehe angerissen wird. Wer daraus Werte
/// haben will, muss es **spielen** - deshalb steht hier ein Abspieler
/// (`xmrsplayer`) und kein Dekodierer.
///
/// Genau das tut SDL_mixer auch; es bindet dafuer libxmp ein.
///
/// Der Abspieler laeuft **einmal** durch: `set_max_loop_count(1)`. Ohne
/// das liefe er endlos, und `Sound(...)` wartete auf ein Ende, das nie
/// kommt. Gemessen an `examples/data/surfonasinewave.xm`.
fn xm_lesen(daten: &[u8]) -> Ergebnis<Klangdaten> {
    modul_lesen(daten, Modulart::Xm)
}

/// Welches Trackerformat.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Modulart {
    Xm,
    Mod,
    S3m,
    It,
}

/// Ein ProTracker-Modul? Die Kennung steht bei Byte 1080 - `M.K.` und
/// Verwandte, wie libxmps `mod_test` sie kennt.
fn ist_mod(d: &[u8]) -> bool {
    let Some(k) = d.get(1080..1084) else {
        return false;
    };
    matches!(
        k,
        b"M.K." | b"M!K!" | b"M&K!" | b"N.T." | b"FLT4" | b"FLT8" | b"4CHN" | b"6CHN" | b"8CHN"
            | b"CD81" | b"OKTA" | b"OCTA" | b"FA04" | b"FA06" | b"FA08"
    ) || (k[0].is_ascii_digit() && k[1].is_ascii_digit() && &k[2..4] == b"CH")
        || (k[0].is_ascii_digit() && k[1].is_ascii_digit() && &k[2..4] == b"CN")
        || (k[0].is_ascii_digit() && &k[1..4] == b"CHN")
}

/// Ein Trackerstueck ganz durchspielen - XM, MOD, S3M oder IT.
///
/// **Ein anderer Abspieler als bei pygame** (xmrs statt libxmp): Laden,
/// Spurzahl und Ablauf stimmen, die Werte nicht - dieselbe Sorte Grenze
/// wie bei OGG und MP3. Scheitert das Lesen, sagt pygame (libxmp hat das
/// Format erkannt) `XMP: Error loading file`.
fn modul_lesen(daten: &[u8], art: Modulart) -> Ergebnis<Klangdaten> {
    use xmrs::prelude::Module;
    use xmrsplayer::prelude::XmrsPlayer;

    let geladen = match art {
        Modulart::Xm => Module::load_xm(daten),
        Modulart::Mod => Module::load_mod(daten),
        Modulart::S3m => Module::load_s3m(daten),
        Modulart::It => Module::load_it(daten),
    };
    let modul = geladen.map_err(|_| Fehler::Format("XMP: Error loading file".into()))?;
    let frequenz = 44_100u32;
    let mut spieler = XmrsPlayer::new(&modul, frequenz, 0);
    spieler.set_max_loop_count(1);
    // Die Werte kommen verschachtelt und in Stereo aus dem Abspieler.
    // Eine Obergrenze ist noetig: ein Stueck mit einem Fehler im
    // Sprungbefehl liefe sonst weiter, bis der Speicher voll ist.
    // Zehn Minuten sind mehr, als je ein Spiel als Hintergrundstueck
    // ablegt.
    let hoechstens = (frequenz as usize) * 2 * 600;
    let werte: Vec<i16> = spieler.by_ref().take(hoechstens).collect();
    Ok(Klangdaten {
        werte,
        spuren: 2,
        frequenz,
    })
}

// ---------------------------------------------------------------------------
//  Opus
// ---------------------------------------------------------------------------

/// Steckt in diesem Ogg ein Opus-Strom?
///
/// Die Kennung `OpusHead` steht im **ersten Paket** der ersten Seite, also
/// gleich hinter dem 27 Byte langen Seitenkopf plus der Tabelle der
/// Stueckgroessen. Statt das nachzurechnen, wird in den ersten paar
/// hundert Bytes danach gesucht - ein Vorbis-Strom traegt dort
/// `\x01vorbis`, und die beiden Woerter kommen einander nicht in die
/// Quere.
fn opus_verdacht(daten: &[u8]) -> bool {
    daten[..daten.len().min(256)]
        .windows(8)
        .any(|f| f == b"OpusHead")
}

/// Liest OPUS aus einem Ogg-Behaelter.
///
/// **Opus rechnet immer in 48 kHz.** Der Kopf nennt zwar eine
/// "Eingangsrate", aber die ist nur eine Notiz darueber, was einmal
/// hineinging; entschluesselt wird in 48 kHz, und pygame gibt genau das
/// heraus. Wer die Notiz fuer die Rate haelt, bekommt ein Stueck, das zu
/// schnell oder zu langsam laeuft - und das faellt beim Hoeren auf,
/// nicht beim Lesen.
///
/// `pre_skip` sind die Werte, die der Kodierer vorne angehaengt hat
/// (seine eigene Verzoegerung); sie gehoeren nicht zum Stueck und werden
/// abgeschnitten.
fn opus_lesen(daten: &[u8]) -> Ergebnis<Klangdaten> {
    let leser = std::io::Cursor::new(daten.to_vec());
    let mut ogg = opus_pure::ogg::OggOpusReader::new(leser)
        .map_err(|e| Fehler::Format(format!("OPUS could not be read: {e}")))?;
    let frequenz = 48_000i32;
    let kopf = ogg.head().clone();
    let spuren = kopf.channel_count as usize;
    let mut dek = kopf
        .decoder(frequenz)
        .map_err(|e| Fehler::Format(format!("OPUS could not be read: {e}")))?;

    // Ein Opus-Paket traegt hoechstens 120 ms, bei 48 kHz also 5760 Werte
    // je Spur.
    let hoechstens = 5760usize;
    let mut puffer = vec![0i16; hoechstens * spuren];
    let mut werte: Vec<i16> = Vec::new();
    while let Ok(Some(paket)) = ogg.read_packet() {
        match dek.decode_s16(&paket.data, hoechstens, &mut puffer) {
            Ok(n) => werte.extend_from_slice(&puffer[..n * spuren]),
            Err(_) => break,
        }
    }
    let weg = (kopf.pre_skip as usize) * spuren;
    if weg < werte.len() {
        werte.drain(..weg);
    }
    Ok(Klangdaten {
        werte,
        spuren: spuren as u16,
        frequenz: frequenz as u32,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn umrechnen_verdoppelt_die_rahmen() {
        let k = Klangdaten {
            werte: vec![0, 100, 200, 300],
            spuren: 1,
            frequenz: 22_050,
        };
        let heraus = umrechnen(&k, 44_100, 1);
        assert_eq!(heraus.len(), 8, "4 Rahmen bei 22050 werden 8 bei 44100");
    }

    #[test]
    fn umrechnen_schneidet_ab_statt_zu_runden() {
        // Gemessen an pygame: 1000 Rahmen bei 48000 Hz ergeben 918 bei
        // 44100 Hz - mathematisch waeren es 918.75.
        let k = Klangdaten {
            werte: vec![0; 1000],
            spuren: 1,
            frequenz: 48_000,
        };
        assert_eq!(umrechnen(&k, 44_100, 1).len(), 918);
        assert_eq!(umrechnen(&k, 44_100, 2).len(), 918 * 2);
    }

    #[test]
    fn mono_wird_auf_beide_spuren_gelegt() {
        let k = Klangdaten {
            werte: vec![10, 20],
            spuren: 1,
            frequenz: 44_100,
        };
        assert_eq!(umrechnen(&k, 44_100, 2), vec![10, 10, 20, 20]);
    }

    /// Stereo nach Mono mittelt - `(a * 0.5) + (b * 0.5)`, wie SDL.
    ///
    /// Der Weg geht jetzt durch `f32` und zurueck, deshalb darf der Wert
    /// um eins danebenliegen; hier trifft er genau.
    #[test]
    fn stereo_wird_zu_mono_gemittelt() {
        let k = Klangdaten {
            werte: vec![10, 30, 100, 200],
            spuren: 2,
            frequenz: 44_100,
        };
        let aus = umrechnen(&k, 44_100, 1);
        assert_eq!(aus.len(), 2);
        assert!((aus[0] - 20).abs() <= 1, "{aus:?}");
        assert!((aus[1] - 150).abs() <= 1, "{aus:?}");
    }

    /// Unbrauchbare MP3-Daten geben einen **Fehler**, keine Panik.
    ///
    /// Hier stand einmal `NichtUnterstuetzt` mit dem Satz "MP3 wird nicht
    /// gelesen" - richtig, solange rodios `mp3`-Merkmal aus war. Seit es an
    /// ist (symphonia, reines Rust), ist die Aussage falsch und der Test
    /// pruefte eine Grenze, die es nicht mehr gibt. Was bleibt, ist die
    /// eigentliche Zusage: der Dekoder darf stolpern, aber nicht umfallen.
    #[test]
    fn unbrauchbares_mp3_gibt_einen_fehler_und_keine_panik() {
        let mut daten = b"ID3\x04\x00\x00\x00\x00\x00\x00".to_vec();
        daten.extend_from_slice(&[0u8; 32]);
        let f = dekodieren(&daten, Weg::Musik).unwrap_err();
        assert!(matches!(f, Fehler::Format(_)), "{f}");
        assert!(f.to_string().contains("MP3"), "{f}");
        let f = dekodieren(&daten, Weg::Klang).unwrap_err();
        assert_eq!(f.to_string(), "Unrecognized audio format");
    }

    /// Ein WavPack- und ein XM-Stueck werden **benannt** abgelehnt.
    ///
    /// Beide nennt `mixer_music_test.py` ausdruecklich, und beide fehlen
    /// hier. Eine benannte Grenze ist etwas anderes als "unbekanntes
    /// Format": sie sagt, dass die Datei erkannt wurde.
    #[test]
    fn wavpack_und_xm_sind_benannte_grenzen() {
        let mut wv = b"wvpk".to_vec();
        wv.extend_from_slice(&[0u8; 32]);
        let f = dekodieren(&wv, Weg::Musik).unwrap_err();
        assert!(f.to_string().contains("WavPack"), "{f}");
        let f = dekodieren(&wv, Weg::Klang).unwrap_err();
        assert_eq!(f.to_string(), "Unrecognized audio format");

        let mut xm = b"Extended Module: ".to_vec();
        xm.extend_from_slice(&[0u8; 32]);
        let f = dekodieren(&xm, Weg::Musik).unwrap_err();
        assert!(f.to_string().contains("XM"), "{f}");
        let f = dekodieren(&xm, Weg::Klang).unwrap_err();
        assert_eq!(f.to_string(), "Unrecognized audio format");
    }

    /// Die kurzen Faelle, gemessen auf beiden Wegen.
    #[test]
    fn unsinn_gibt_pygames_meldung() {
        let satz = |d: &[u8], w| dekodieren(d, w).unwrap_err().to_string();
        assert_eq!(satz(b"kurz", Weg::Musik), "Couldn't read first 12 bytes of audio data");
        assert_eq!(satz(b"kurz", Weg::Klang), "Couldn't read first 12 bytes of audio data");
        assert_eq!(satz(b"", Weg::Klang), "Couldn't read first 4 bytes of audio data");
        assert_eq!(satz(b"RI", Weg::Klang), "Couldn't read first 4 bytes of audio data");
        assert_eq!(satz(b"RIFF", Weg::Klang), "Could not read RIFF header");
        assert_eq!(satz(b"RIFF\0\0\0\0WAV", Weg::Klang), "Could not read RIFF form type");
        assert_eq!(satz(b"RIFF\0\0\0\0WAV", Weg::Musik), "Couldn't read first 12 bytes of audio data");
        assert_eq!(
            satz(b"RIFF\x04\0\0\0AVI ", Weg::Klang),
            "RIFF form type is not WAVE (not a Waveform file)"
        );
        assert_eq!(satz(b"abcdefghijklmnop", Weg::Klang), "Unrecognized audio format");
    }

    #[test]
    fn copy_braucht_einen_laufenden_mixer() {
        // `snd_copy` faengt mit `MIXER_INIT_CHECK()` an - gemessen gibt
        // `s.copy()` nach `mixer.quit()` "mixer not initialized".
        crate::mixer::set_treiber(crate::treiber::Wahl::Dummy);
        crate::mixer::init(Default::default()).unwrap();
        let s = Sound::from_samples(&[100i16; 8], 44_100, 1).unwrap();
        crate::mixer::quit();
        assert_eq!(s.copy().unwrap_err(), Fehler::NichtInitialisiert);
    }

    #[test]
    fn copy_ist_eine_eigene_kopie_mit_derselben_lautstaerke() {
        crate::mixer::set_treiber(crate::treiber::Wahl::Dummy);
        crate::mixer::init(Default::default()).unwrap();
        let s = Sound::from_samples(&[100i16; 8], 44_100, 1).unwrap();
        s.set_volume(0.3);
        let k = s.copy().unwrap();

        // Gemessen: die Lautstaerke wird mitgenommen (0.3 rastet auf
        // 38/128 = 0.296875 ein), die Daten sind gleich, das Objekt ist
        // ein anderes - `s.copy() == s` ist in pygame `False`.
        assert_eq!(k.get_volume(), s.get_volume());
        assert_eq!(k.get_volume(), 0.296_875);
        assert_eq!(k.get_raw(), s.get_raw());
        assert_eq!(k.get_length(), s.get_length());
        assert_ne!(k, s);

        // Und die Kopie ist wirklich eigenstaendig - anders als ein
        // `clone()`, das absichtlich denselben Klang zurueckgibt.
        k.set_volume(0.9);
        assert_eq!(s.get_volume(), 0.296_875);
        assert_eq!(k.get_volume(), 0.898_437_5);
        let zwilling = s.clone();
        zwilling.set_volume(0.1);
        assert_eq!(s.get_volume(), zwilling.get_volume());
        crate::mixer::quit();
    }

    #[test]
    fn get_raw_haengt_am_ausgabeformat() {
        // Gemessen an echtem pygame mit `punch.wav` (16164 Rahmen, zwei
        // Spuren): `get_raw()` ist unter `-8`/`8` **halb so lang** wie
        // unter `-16`/`16`, `get_length()` bleibt gleich.
        crate::mixer::set_treiber(crate::treiber::Wahl::Dummy);
        for (groesse, je_wert) in [(-16, 2), (16, 2), (-8, 1), (8, 1), (-32, 4)] {
            crate::mixer::quit();
            crate::mixer::init(crate::mixer::Einstellungen::neu(44_100, groesse, 2, 0)).unwrap();
            let s = Sound::from_samples(&[1000i16; 40], 44_100, 2).unwrap();
            assert_eq!(s.get_raw().len(), 40 * je_wert, "groesse={groesse}");
            assert_eq!(s.get_length(), (20.0f32 / 44_100.0) as f64);
        }
        crate::mixer::quit();
    }

    #[test]
    fn roh_hin_und_zurueck_verliert_nichts() {
        // Darauf baut `Sound.copy()`: es schiebt die Bytes durch
        // `Sound(buffer=...)`, so wie `snd_copy` in `src_c/mixer.c`.
        let werte = [-32_768i16, -256, -1, 0, 1, 255, 256, 32_512];
        for groesse in [-16, 16, -8, 8, -32] {
            let bytes = roh_schreiben(&werte, groesse);
            assert_eq!(bytes.len(), werte.len() * bytes_je_wert(groesse));
            let zurueck = roh_lesen(&bytes, groesse);
            if groesse.abs() == 8 {
                // Acht Bit koennen nur das obere Byte halten - geprueft
                // wird, dass genau das erhalten bleibt.
                for (a, b) in werte.iter().zip(zurueck.iter()) {
                    assert_eq!(a >> 8, b >> 8, "groesse={groesse}, wert={a}");
                }
            } else {
                assert_eq!(zurueck, werte, "groesse={groesse}");
            }
        }
    }

    #[test]
    fn copy_laeuft_getrennt_vom_original() {
        crate::mixer::set_treiber(crate::treiber::Wahl::Dummy);
        crate::mixer::init(Default::default()).unwrap();
        let s = Sound::from_samples(&[100i16; 44_100], 44_100, 1).unwrap();
        let k = s.copy().unwrap();
        s.play(0, 0, 0).unwrap();
        // Gemessen: nach `s.play()` zaehlt die Kopie 0 Kanaele.
        assert_eq!(s.get_num_channels(), 1);
        assert_eq!(k.get_num_channels(), 0);
        crate::mixer::quit();
    }
}
