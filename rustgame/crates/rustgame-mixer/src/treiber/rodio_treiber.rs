//! treiber/rodio_treiber.rs
//! ========================
//! Der Treiber, der wirklich Schall macht - ueber `rodio`.
//!
//! Je Stimme eine `Sink`. Das ist nicht die sparsamste Loesung (SDL_mixer
//! mischt alles selbst in einen Puffer), aber die ehrlichste: eine `Sink`
//! kann genau das, was ein pygame-Kanal koennen muss - anhalten,
//! weiterlaufen, Lautstaerke aendern, abbrechen - und rodio mischt die
//! Sinks ohnehin zusammen.
//!
//! ## Was hier nicht passiert
//!
//! Gefragt wird die `Sink` nie. Ob ein Kanal belegt ist, steht in
//! [`crate::zustand`] und wird aus der Uhr gerechnet; `Sink::empty()`
//! kaeme zwar ungefaehr aufs Gleiche heraus, aber eben nur ungefaehr -
//! und dann liefe der stille Treiber anders als dieser. Genau das soll
//! nicht sein.

use std::collections::HashMap;
use std::time::Duration;

use rodio::buffer::SamplesBuffer;
use rodio::{OutputStream, OutputStreamHandle, Sink, Source};

use crate::treiber::{Auftrag, Stimme, Treiber};
use crate::{Ergebnis, Fehler};

/// Ton ueber das Tongeraet des Rechners.
pub struct RodioTreiber {
    /// Muss am Leben bleiben, sonst verstummt alles - der Strom haengt
    /// daran, nicht am Griff.
    _strom: OutputStream,
    griff: OutputStreamHandle,
    sinks: HashMap<Stimme, Sink>,
    name: String,
}

impl RodioTreiber {
    /// Oeffnet das Standard-Tongeraet.
    ///
    /// Gibt [`Fehler::KeinGeraet`] zurueck, wenn keines da ist - der Fall,
    /// den diese Kiste ausdruecklich sauber behandeln muss.
    pub fn neu() -> Ergebnis<RodioTreiber> {
        let (strom, griff) = OutputStream::try_default()
            .map_err(|e| Fehler::KeinGeraet(format!("no audio device: {e}")))?;
        Ok(RodioTreiber {
            _strom: strom,
            griff,
            sinks: HashMap::new(),
            name: "rodio".to_string(),
        })
    }

    /// Holt die Sink zur Stimme oder legt eine an.
    fn sink(&mut self, stimme: Stimme) -> Option<&Sink> {
        if !self.sinks.contains_key(&stimme) {
            let s = Sink::try_new(&self.griff).ok()?;
            self.sinks.insert(stimme, s);
        }
        self.sinks.get(&stimme)
    }
}

impl Treiber for RodioTreiber {
    fn name(&self) -> &str {
        &self.name
    }

    fn spielen(&mut self, stimme: Stimme, auftrag: Auftrag<'_>) {
        if auftrag.werte.is_empty() || auftrag.spuren == 0 || auftrag.frequenz == 0 {
            return;
        }
        // Die Werte muessen kopiert werden: die Quelle wandert auf den
        // Tonstrang und braucht deshalb Eigentum.
        let puffer = SamplesBuffer::new(auftrag.spuren, auftrag.frequenz, auftrag.werte.to_vec());
        let eine_laenge = puffer.total_duration().unwrap_or_default();

        let mut quelle: Box<dyn Source<Item = i16> + Send> = Box::new(puffer);
        if auftrag.ab_ms > 0 {
            quelle = Box::new(quelle.skip_duration(Duration::from_millis(auftrag.ab_ms)));
        }
        match auftrag.durchlaeufe {
            0 => return,
            1 => {}
            d if d < 0 => quelle = Box::new(quelle.repeat_infinite()),
            d => {
                // rodio kann nur endlos wiederholen - also endlos
                // wiederholen und nach der richtigen Zeit abschneiden.
                let gesamt = eine_laenge
                    .saturating_mul(d as u32)
                    .saturating_sub(Duration::from_millis(auftrag.ab_ms));
                quelle = Box::new(quelle.repeat_infinite().take_duration(gesamt));
            }
        }
        if let Some(h) = auftrag.hoechstdauer_ms {
            quelle = Box::new(quelle.take_duration(Duration::from_millis(h)));
        }
        if auftrag.einblenden_ms > 0 {
            quelle = Box::new(quelle.fade_in(Duration::from_millis(auftrag.einblenden_ms)));
        }

        if let Some(sink) = self.sink(stimme) {
            sink.clear();
            sink.set_volume(auftrag.lautstaerke.clamp(0.0, 1.0));
            sink.append(quelle);
            sink.play();
        }
    }

    fn stoppen(&mut self, stimme: Stimme) {
        if let Some(sink) = self.sinks.get(&stimme) {
            // `clear` statt `stop`: `stop` macht die Sink unbrauchbar,
            // `clear` laesst sie fuer den naechsten Klang stehen.
            sink.clear();
        }
    }

    fn pausieren(&mut self, stimme: Stimme) {
        if let Some(sink) = self.sinks.get(&stimme) {
            sink.pause();
        }
    }

    fn fortsetzen(&mut self, stimme: Stimme) {
        if let Some(sink) = self.sinks.get(&stimme) {
            sink.play();
        }
    }

    fn lautstaerke(&mut self, stimme: Stimme, wert: f32) {
        if let Some(sink) = self.sinks.get(&stimme) {
            sink.set_volume(wert.clamp(0.0, 1.0));
        }
    }

    fn kanaele_setzen(&mut self, anzahl: usize) {
        self.sinks.retain(|s, _| match s {
            Stimme::Musik => true,
            Stimme::Kanal(i) => *i < anzahl,
        });
    }
}

