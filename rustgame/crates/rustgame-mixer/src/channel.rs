//! channel.rs
//! ==========
//! `pygame.mixer.Channel` - ein Mischkanal.
//!
//! Ein `Channel` ist nur ein **Platzhalter fuer einen Index**, kein
//! Besitzer von irgendetwas. Der Zustand liegt in `crate::zustand`;
//! `Channel(3)` ist derselbe Kanal, egal wie oft man ihn herstellt.
//!
//! Deshalb ist der Typ `Copy` und vergleicht sich ueber den Index. In
//! Python ist das anders - dort gibt `pygame.mixer.Channel(0) ==
//! pygame.mixer.Channel(0)` gemessen **`False`**, weil die Klasse kein
//! `__eq__` hat und Python auf Objektgleichheit zurueckfaellt. Das ist
//! eine Eigenart der Sprache, keine des Mixers, und wird hier nicht
//! nachgebaut.
//!
//! ## Die Lautstaerke mit zwei Werten ist eine Falle
//!
//! [`Channel::set_volume_stereo`] stellt **nicht** zwei Lautstaerken ein.
//! Es stellt die Balance ein und setzt die Lautstaerke dabei auf voll -
//! und die Balance laesst sich danach nirgends mehr auslesen. Das steht
//! so in pygames Quelltext (`chan_set_volume`, mit dem Kommentar "NOTE:
//! there is no way to GetPanning variables") und ist genau so gemessen:
//!
//! ```text
//! c.set_volume(0.3, 0.7)  ->  c.get_volume() == 1.0
//! c.set_volume(0.25, 0.25) ->  c.get_volume() == 1.0
//! ```

use crate::konstanten::*;
use crate::sound::{anwerfen, Sound};
use crate::zustand::{self, Zustand};
use crate::{Ergebnis, Fehler};

/// Ein Mischkanal - `pygame.mixer.Channel`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Channel(usize);

impl Channel {
    /// Der Kanal mit diesem Index - `pygame.mixer.Channel(n)`.
    ///
    /// Gemessen: pygame wirft `IndexError: invalid channel index`, wenn
    /// der Index nicht kleiner ist als `get_num_channels()`. Negative
    /// Zahlen kann es hier nicht geben; pygame lehnt auch `-1` ab.
    pub fn new(index: usize) -> Ergebnis<Channel> {
        zustand::mit(|z| {
            z.pruefen()?;
            if index >= z.anzahl {
                return Err(Fehler::UngueltigerKanal(index));
            }
            Ok(Channel(index))
        })
    }

    /// Ohne Pruefung - nur fuer die Kiste selbst.
    pub(crate) fn roh(index: usize) -> Channel {
        Channel(index)
    }

    /// Der Index - `Channel.id`.
    pub fn id(&self) -> usize {
        self.0
    }

    // -- Spielen ----------------------------------------------------------

    /// Spielt einen Klang auf **diesem** Kanal - `Channel.play(...)`.
    ///
    /// `loops` zaehlt wie bei [`Sound::play`] die Wiederholungen:
    /// gemessen spielt `loops=1` zweimal.
    ///
    /// Anders als [`Sound::play`] bleibt die eingestellte Lautstaerke des
    /// Kanals stehen - gemessen: nach `c.set_volume(0.25); c.play(s)` ist
    /// `c.get_volume()` weiter `0.25`, waehrend `s.play()` den benutzten
    /// Kanal auf `1.0` zurueckdreht.
    pub fn play(&self, sound: &Sound, loops: i32, maxtime: i32, fade_ms: i32) -> Ergebnis<()> {
        zustand::mit(|z| {
            self.pruefen(z)?;
            anwerfen(z, self.0, sound, loops, maxtime, fade_ms);
            Ok(())
        })
    }

    /// Haengt einen Klang hinten an - `Channel.queue(sound)`.
    ///
    /// Laeuft gerade nichts, faengt er sofort an - gemessen: `get_queue()`
    /// ist danach `None` und `get_sound()` ist der Klang selbst.
    /// Andernfalls ersetzt er, was schon in der Schlange stand; es passt
    /// immer nur **einer** hinein.
    pub fn queue(&self, sound: &Sound) -> Ergebnis<()> {
        zustand::mit(|z| {
            self.pruefen(z)?;
            if z.kanaele[self.0].lauf.is_none() {
                anwerfen(z, self.0, sound, 0, 0, 0);
            } else {
                z.kanaele[self.0].schlange = Some(sound.clone());
            }
            Ok(())
        })
    }

    /// Was in der Schlange steht - `Channel.get_queue()`.
    pub fn get_queue(&self) -> Option<Sound> {
        zustand::mit(|z| z.kanaele.get(self.0).and_then(|k| k.schlange.clone()))
    }

    /// Was gerade laeuft - `Channel.get_sound()`.
    ///
    /// Gemessen: nach dem Ende ist das `None`, nicht der letzte Klang.
    pub fn get_sound(&self) -> Option<Sound> {
        zustand::mit(|z| z.kanaele.get(self.0).and_then(|k| k.sound.clone()))
    }

    /// Laeuft hier etwas - `Channel.get_busy()`.
    ///
    /// Gemessen und ueberraschend: ein **pausierter** Kanal gilt weiter
    /// als belegt. `c.play(s); c.pause(); c.get_busy()` ist `True`.
    /// SDL_mixers `Mix_Playing` zaehlt Pausen mit, und pygame reicht das
    /// unveraendert durch.
    pub fn get_busy(&self) -> bool {
        zustand::mit(|z| z.kanaele.get(self.0).is_some_and(|k| k.lauf.is_some()))
    }

    /// Bricht ab - `Channel.stop()`.
    ///
    /// **Achtung, gemessen:** wenn eine Schlange gesetzt ist, faengt die
    /// dabei an zu spielen. Nach
    ///
    /// ```text
    /// c.play(lang); c.queue(kurz); c.stop()
    /// ```
    ///
    /// ist `c.get_busy()` in echtem pygame `True` und `c.get_sound()` ist
    /// `kurz`. Der Grund steht in `src_c/mixer.c`: SDL ruft den
    /// Fertig-Rueckruf auch beim Abbrechen, und der laesst die Schlange
    /// aufruecken. Ausserdem faellt dabei das Endereignis an - ebenfalls
    /// gemessen.
    pub fn stop(&self) {
        zustand::mit(|z| {
            if self.pruefen(z).is_ok() {
                let jetzt = rustgame_core::time::get_ticks();
                z.kanal_beenden(self.0, jetzt);
            }
        });
    }

    /// Haelt an, ohne die Stelle zu verlieren - `Channel.pause()`.
    pub fn pause(&self) {
        zustand::mit(|z| pausieren(z, self.0, true));
    }

    /// Macht weiter - `Channel.unpause()`.
    pub fn unpause(&self) {
        zustand::mit(|z| pausieren(z, self.0, false));
    }

    /// Blendet aus und hoert dann auf - `Channel.fadeout(ms)`.
    ///
    /// Wie [`Channel::stop`] loest auch das am Ende das Endereignis aus
    /// und laesst die Schlange aufruecken.
    pub fn fadeout(&self, millisekunden: i32) {
        zustand::mit(|z| {
            let jetzt = rustgame_core::time::get_ticks();
            ausblenden_setzen(z, self.0, millisekunden, jetzt);
        });
    }

    // -- Lautstaerke ------------------------------------------------------

    /// Stellt die Lautstaerke ein - `Channel.set_volume(wert)`.
    ///
    /// Setzt die Balance dabei auf Mitte zurueck, falls sie vorher mit
    /// [`Channel::set_volume_stereo`] verstellt war - so steht es in
    /// pygames Quelltext, und anders koennte man die Balance nie wieder
    /// loswerden.
    ///
    /// Wie ueberall: 128 Stufen, ueber 1.0 wird gekappt, negativ aendert
    /// nichts.
    pub fn set_volume(&self, wert: f64) {
        zustand::mit(|z| {
            let Some(k) = z.kanaele.get_mut(self.0) else {
                return;
            };
            k.panning = (MAX_PANNING, MAX_PANNING);
            k.lautstaerke = crate::mixer::stufe(wert, k.lautstaerke);
            lautstaerke_melden(z, self.0);
        });
    }

    /// Stellt die **Balance** ein - `Channel.set_volume(links, rechts)`.
    ///
    /// Trotz des Namens ist das keine Lautstaerke je Seite, sondern
    /// `Mix_SetPanning` - und die Lautstaerke des Kanals springt dabei auf
    /// voll. Gemessen liefert `get_volume()` danach **immer** `1.0`, egal
    /// welche Werte man uebergibt, und die eingestellte Balance laesst
    /// sich nirgends mehr abfragen.
    pub fn set_volume_stereo(&self, links: f64, rechts: f64) {
        zustand::mit(|z| {
            let Some(k) = z.kanaele.get_mut(self.0) else {
                return;
            };
            k.panning = (
                (links as f32 * MAX_PANNING as f32).clamp(0.0, MAX_PANNING as f32) as i32,
                (rechts as f32 * MAX_PANNING as f32).clamp(0.0, MAX_PANNING as f32) as i32,
            );
            k.lautstaerke = MAX_VOLUME;
            lautstaerke_melden(z, self.0);
        });
    }

    /// Die Lautstaerke von 0.0 bis 1.0 - `Channel.get_volume()`.
    pub fn get_volume(&self) -> f64 {
        zustand::mit(|z| {
            z.kanaele
                .get(self.0)
                .map_or(0.0, |k| k.lautstaerke as f64 / MAX_VOLUME as f64)
        })
    }

    // -- Ort ---------------------------------------------------------------

    /// Stellt ein, **woher** der Klang kommt -
    /// `Channel.set_source_location(angle, distance)`.
    ///
    /// `angle` ist die Richtung in Grad, im Uhrzeigersinn, `0` geradeaus.
    /// `distance` ist die Entfernung von `0` (ganz nah) bis `255` (ganz
    /// weit weg, still).
    ///
    /// ## Die Reihenfolge der Pruefungen ist die von pygame
    ///
    /// In `chan_set_source_location` (`src_c/mixer.c`) steht das
    /// `MIXER_INIT_CHECK()` **hinter** der Bereichspruefung. Gemessen an
    /// echtem pygame nach `mixer.quit()`:
    ///
    /// ```text
    /// ch.set_source_location(0, 999)  ->  ValueError: distance out of range, expected (0, 255)
    /// ch.set_source_location(0, 1)    ->  pygame.error: mixer not initialized
    /// ```
    ///
    /// Also entscheidet hier `distance` zuerst. Genauso gemessen: es wird
    /// **gerundet**, nicht abgeschnitten - `255.4` geht durch, `255.5`
    /// nicht, `-0.4` geht durch, `-0.5` nicht.
    ///
    /// ## Was davon zu hoeren ist
    ///
    /// `Mix_SetPosition` legt zwei Dinge ab (`effect_position.c`): eine
    /// Seitenverteilung aus dem Winkel und einen Daempfungsfaktor
    /// `(255 - distance) / 255`. Der Faktor wirkt hier wirklich - er geht
    /// in die Lautstaerke des Kanals ein. Die Seitenverteilung wird
    /// gemerkt, aber nicht gespielt; dem Treiber fehlt dafuer die zweite
    /// Seite (siehe [`crate::treiber::Treiber::lautstaerke`]).
    ///
    /// `Channel.get_volume()` bleibt davon unberuehrt - gemessen: nach
    /// `set_volume(0.25); set_source_location(45, 200)` steht dort weiter
    /// `0.25`.
    ///
    /// `set_source_location(0, 0)` meldet den Effekt in SDL wieder ab;
    /// hier stellt es Seitenverteilung und Entfernung auf "nichts zu
    /// tun" zurueck.
    pub fn set_source_location(&self, angle: f32, distance: f32) -> Ergebnis<()> {
        // `distance_f = roundf(distance_f); if (0 > distance_f || 256 <=
        // distance_f) ...` - Zeile fuer Zeile uebertragen. Rusts
        // `f32::round` rundet wie C's `roundf` von der Null weg.
        let nah = distance.round();
        if 0.0 > nah || 256.0 <= nah {
            return Err(Fehler::Wert(
                "distance out of range, expected (0, 255)".into(),
            ));
        }
        // NaN faellt durch beide Vergleiche - und gemessen laesst echtes
        // pygame `set_source_location(0, float('nan'))` tatsaechlich
        // durch. In C wird daraus ein unbestimmter `Uint8`; Rusts `as`
        // macht daraus 0, und mehr Festlegung gibt es dort nicht.
        let nah = nah as i32;

        // `angle = (Sint16)roundf(fmodf(angle_f, 360))`, danach in
        // `Mix_SetPosition` noch `angle %= 360; if (angle < 0) angle +=
        // 360;`. `inf` und `nan` ergeben in Rust 0 - gemessen nimmt
        // pygame beide ebenfalls ohne Fehler an.
        let winkel = ((angle % 360.0).round() as i32).rem_euclid(360);

        zustand::mit(|z| {
            self.pruefen(z)?;
            let spuren = z.offen.map_or(2, |s| s.spuren);
            let k = &mut z.kanaele[self.0];
            if winkel == 0 && nah == 0 {
                // `if ((!distance) && (!angle)) ... _Mix_UnregisterEffect`
                k.panning = (MAX_PANNING, MAX_PANNING);
                k.entfernung = MAX_PANNING;
            } else {
                k.panning = amplituden(spuren, winkel);
                // `distance = 255 - distance;` - SDL dreht die Zahl um,
                // bevor es sie ablegt.
                k.entfernung = MAX_PANNING - nah;
            }
            lautstaerke_melden(z, self.0);
            Ok(())
        })
    }

    // -- Endereignis ------------------------------------------------------

    /// Welches Ereignis das Ende melden soll - `Channel.set_endevent()`.
    ///
    /// [`crate::konstanten::NOEVENT`] (also `0`) schaltet es ab; das ist
    /// pygames `set_endevent()` ohne Argument. Wo die Ereignisse
    /// hinterher landen, steht in [`crate::ereignis`].
    pub fn set_endevent(&self, typ: u32) {
        zustand::mit(|z| {
            if let Some(k) = z.kanaele.get_mut(self.0) {
                k.endevent = typ;
            }
        });
    }

    /// Welches Ereignis gesetzt ist - `Channel.get_endevent()`.
    ///
    /// Gemessen: ohne vorheriges Setzen ist das `0`.
    pub fn get_endevent(&self) -> u32 {
        zustand::mit(|z| z.kanaele.get(self.0).map_or(NOEVENT, |k| k.endevent))
    }

    // -- innen ------------------------------------------------------------

    fn pruefen(&self, z: &Zustand) -> Ergebnis<()> {
        z.pruefen()?;
        if self.0 >= z.anzahl {
            return Err(Fehler::UngueltigerKanal(self.0));
        }
        Ok(())
    }
}

/// Die Seitenverteilung zu einem Winkel, wie `set_amplitudes` in
/// SDL_mixers `effect_position.c`.
///
/// Uebertragen ist **nur** der Zweig fuer zwei Ausgabespuren; fuer 1, 4
/// und 6 bleibt es bei `(255, 255)`. Das ist keine Nachlaessigkeit,
/// sondern die Grenze des Nutzens: die Zahlen lassen sich weder aus
/// pygame auslesen (`get_volume()` zeigt sie nicht, und einen Getter gibt
/// es nicht - so steht es woertlich in `chan_set_volume`) noch hier
/// hoeren, weil der Treiber einen einzigen Lautstaerkeregler hat. Was
/// wirklich wirkt, ist die Entfernung, und die haengt nicht am Winkel.
///
/// Der Kommentar in der Quelle nennt den Gedanken dahinter: gedaempft
/// wird erst, wenn der Winkel auf die **abgewandte** Seite faellt - "I
/// think of this as occlusion by one's own head".
fn amplituden(spuren: i32, winkel: i32) -> (i32, i32) {
    if spuren != 2 {
        return (MAX_PANNING, MAX_PANNING);
    }
    // `(int) (255.0f * (((float) x) / 89.0f))` - in einfacher
    // Genauigkeit und gegen null abgeschnitten, wie in C.
    let teil = |x: i32| (255.0f32 * (x as f32 / 89.0f32)) as i32;
    let (mut links, mut rechts) = (MAX_PANNING, MAX_PANNING);
    if winkel < 90 {
        links = MAX_PANNING - teil(winkel);
    } else if winkel < 180 {
        links = teil(winkel - 90);
    } else if winkel < 270 {
        rechts = MAX_PANNING - teil(winkel - 180);
    } else {
        rechts = teil(winkel - 270);
    }
    let links = links.clamp(0, MAX_PANNING);
    let rechts = rechts.clamp(0, MAX_PANNING);
    // `if (angle > 180) room_angle = 180;` - und bei 180 Grad tauscht
    // `set_amplitudes` die beiden Seiten.
    if winkel > 180 {
        (rechts, links)
    } else {
        (links, rechts)
    }
}

/// Schickt die neue Lautstaerke an den Treiber, falls gerade etwas laeuft.
fn lautstaerke_melden(z: &mut Zustand, i: usize) {
    let jetzt = rustgame_core::time::get_ticks();
    let Some(k) = z.kanaele.get(i) else {
        return;
    };
    if k.lauf.is_none() {
        return;
    }
    let f = k.faktor(jetzt);
    if let Some(t) = z.treiber.as_mut() {
        t.lautstaerke(crate::treiber::Stimme::Kanal(i), f);
    }
}

/// Pausiert oder setzt fort.
fn pausieren(z: &mut Zustand, i: usize, an: bool) {
    let jetzt = rustgame_core::time::get_ticks();
    let Some(k) = z.kanaele.get_mut(i) else {
        return;
    };
    let Some(lauf) = k.lauf.as_mut() else {
        return;
    };
    match (an, lauf.pause_seit) {
        (true, None) => {
            lauf.pause_seit = Some(jetzt);
            if let Some(t) = z.treiber.as_mut() {
                t.pausieren(crate::treiber::Stimme::Kanal(i));
            }
        }
        (false, Some(seit)) => {
            lauf.pause_summe += jetzt.saturating_sub(seit);
            lauf.pause_seit = None;
            if let Some(t) = z.treiber.as_mut() {
                t.fortsetzen(crate::treiber::Stimme::Kanal(i));
            }
        }
        _ => {}
    }
}

/// Setzt eine Ausblendung - gemeinsam fuer [`Channel::fadeout`],
/// [`Sound::fadeout`] und `mixer.fadeout()`.
///
/// Eine Dauer von `0` oder weniger heisst: sofort Schluss.
pub(crate) fn ausblenden_setzen(z: &mut Zustand, i: usize, millisekunden: i32, jetzt: u64) {
    if i >= z.anzahl {
        return;
    }
    let Some(lauf) = z.kanaele[i].lauf.as_mut() else {
        return;
    };
    if millisekunden <= 0 {
        z.kanal_beenden(i, jetzt);
        return;
    }
    let ab = lauf.verstrichen(jetzt);
    lauf.ausblenden = Some((ab, millisekunden as u64));
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mixer;
    use crate::treiber::Wahl;

    /// Stiller Treiber an, Mixer hoch - **kein Test oeffnet ein
    /// Tongeraet.** Unter `cargo test` waehlt `treiber::oeffnen` den
    /// stillen Treiber ohnehin; die Zeile steht trotzdem da, weil dieselbe
    /// Vorbereitung in `tests/gegen_pygame.rs` noetig ist und man sie sonst
    /// beim Umziehen eines Tests vergisst.
    fn mit_mixer(f: impl FnOnce()) {
        mixer::set_treiber(Wahl::Dummy);
        mixer::init(Default::default()).unwrap();
        f();
        mixer::quit();
    }

    #[test]
    fn amplituden_wie_in_effect_position_c() {
        // Von Hand nachgerechnet aus `set_amplitudes`, Zweig `channels == 2`.
        assert_eq!(amplituden(2, 0), (255, 255)); // geradeaus: nichts gedaempft
        assert_eq!(amplituden(2, 90), (0, 255)); // ganz rechts: links stumm
        // 270 Grad sieht spiegelverkehrt aus, und das ist richtig so: die
        // Rechnung ergibt dort `left = 255, right = 0`, aber wegen
        // `room_angle == 180` legt `set_amplitudes` die beiden getauscht
        // ab. Was hier steht, ist `speaker_amplitude[0..1]` - also das
        // Ergebnis **nach** dem Tausch, so wie der Effekt es benutzt.
        assert_eq!(amplituden(2, 270), (0, 255));
        // 180 Grad liegt noch im dritten Zweig und wird nicht getauscht
        // (`if (angle > 180)`, nicht `>=`).
        assert_eq!(amplituden(2, 180), (255, 255));
        // Ein Grad weiter schon: 255 - (int)(255 * 1/89) = 253.
        assert_eq!(amplituden(2, 181), (253, 255));
        assert_eq!(amplituden(2, 179), (255, 255));
        // Mono und Raumklang bekommen hier nichts; siehe [`amplituden`].
        assert_eq!(amplituden(1, 90), (255, 255));
        assert_eq!(amplituden(6, 90), (255, 255));
    }

    #[test]
    fn set_source_location_prueft_die_entfernung_vor_dem_mixer() {
        // Gemessen an echtem pygame nach `mixer.quit()`: der ValueError
        // kommt, obwohl der Mixer gar nicht laeuft.
        let c = Channel::roh(0);
        assert_eq!(
            c.set_source_location(0.0, 999.0).unwrap_err(),
            Fehler::Wert("distance out of range, expected (0, 255)".into())
        );
        // Mit brauchbarer Entfernung kommt dagegen der Mixerfehler.
        assert_eq!(
            c.set_source_location(0.0, 1.0).unwrap_err(),
            Fehler::NichtInitialisiert
        );
    }

    #[test]
    fn set_source_location_rundet_die_entfernung() {
        mit_mixer(|| {
            let c = Channel::new(0).unwrap();
            // Alle vier Zeilen an echtem pygame gemessen.
            assert!(c.set_source_location(0.0, 255.4).is_ok());
            assert!(c.set_source_location(0.0, 255.5).is_err());
            assert!(c.set_source_location(0.0, -0.4).is_ok());
            assert!(c.set_source_location(0.0, -0.5).is_err());
            // NaN kommt durch, unendlich nicht - ebenfalls gemessen.
            assert!(c.set_source_location(0.0, f32::NAN).is_ok());
            assert!(c.set_source_location(0.0, f32::INFINITY).is_err());
            // Der Winkel wird nie abgelehnt.
            for w in [370.0, -10.0, 1e9, f32::NAN, f32::INFINITY] {
                assert!(c.set_source_location(w, 10.0).is_ok(), "Winkel {w}");
            }
        });
    }

    #[test]
    fn set_source_location_laesst_get_volume_stehen() {
        mit_mixer(|| {
            let c = Channel::new(0).unwrap();
            c.set_volume(0.25);
            c.set_source_location(45.0, 200.0).unwrap();
            // Gemessen: 0.25, nicht 1.0 und nicht gedaempft.
            assert_eq!(c.get_volume(), 0.25);
        });
    }

    #[test]
    fn entfernung_daempft_den_treiber_und_null_nimmt_sie_zurueck() {
        mit_mixer(|| {
            let c = Channel::new(0).unwrap();
            let fern = || zustand::mit(|z| z.kanaele[0].entfernung);
            let faktor = || zustand::mit(|z| z.kanaele[0].faktor(0));
            assert_eq!(fern(), MAX_PANNING);
            assert_eq!(faktor(), 1.0);

            // `distance = 255 - distance` aus `Mix_SetPosition`.
            c.set_source_location(0.0, 255.0).unwrap();
            assert_eq!(fern(), 0);
            assert_eq!(faktor(), 0.0);

            c.set_source_location(0.0, 51.0).unwrap();
            assert_eq!(fern(), 204);
            assert_eq!(faktor(), 0.8);

            // `(!distance) && (!angle)` meldet den Effekt wieder ab.
            c.set_source_location(0.0, 0.0).unwrap();
            assert_eq!(fern(), MAX_PANNING);
            assert_eq!(c.get_volume(), 1.0);
        });
    }

    #[test]
    fn set_source_location_setzt_die_seitenverteilung() {
        mit_mixer(|| {
            let c = Channel::new(0).unwrap();
            c.set_source_location(90.0, 10.0).unwrap();
            assert_eq!(zustand::mit(|z| z.kanaele[0].panning), (0, 255));
            // 450 Grad ist derselbe Winkel wie 90.
            c.set_source_location(450.0, 10.0).unwrap();
            assert_eq!(zustand::mit(|z| z.kanaele[0].panning), (0, 255));
            // -90 Grad wird zu 270 - siehe den Tausch in [`amplituden`].
            c.set_source_location(-90.0, 10.0).unwrap();
            assert_eq!(zustand::mit(|z| z.kanaele[0].panning), (0, 255));
            // `set_volume` stellt die Verteilung wieder auf Mitte -
            // pygames `chan_set_volume` ruft `Mix_SetPanning(255, 255)`.
            c.set_volume(0.5);
            assert_eq!(zustand::mit(|z| z.kanaele[0].panning), (255, 255));
            // Die Entfernung bleibt dabei stehen: `Mix_SetPanning` meldet
            // den Effekt nur ab, wenn `distance_u8` schon 255 ist.
            assert_eq!(zustand::mit(|z| z.kanaele[0].entfernung), 245);
        });
    }

    #[test]
    fn set_source_location_lehnt_fremde_kanaele_ab() {
        mit_mixer(|| {
            assert_eq!(
                Channel::roh(8).set_source_location(0.0, 0.0).unwrap_err(),
                Fehler::UngueltigerKanal(8)
            );
        });
    }
}
