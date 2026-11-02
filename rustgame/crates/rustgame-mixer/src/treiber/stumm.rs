//! treiber/stumm.rs
//! ================
//! Der Treiber **ohne Ton** - und der eigentliche Betriebsfall fuer alles,
//! was kopflos laeuft.
//!
//! Dass hier so wenig steht, ist kein Mangel, sondern der Beweis: die
//! gesamte beobachtbare Buchfuehrung - Lautstaerke, Laufzeit,
//! Kanalbelegung, Warteschlangen - liegt in [`crate::zustand`] und nicht
//! im Treiber. Dieses Stueck muss deshalb nichts mitfuehren, um sich
//! richtig zu verhalten: es kann gar nicht abweichen.
//!
//! Dass die Auftraege trotzdem mitgeschrieben werden, hat einen anderen
//! Grund - so laesst sich pruefen, **dass** etwas abgeschickt wurde,
//! obwohl nichts zu hoeren ist. Das ist dasselbe Werkzeug wie
//! `rustgame_display::backend::dummy::bildzaehler`.
//!
//! Eingeschaltet wird er mit `RUSTGAME_AUDIODRIVER=dummy`, mit
//! [`crate::mixer::set_treiber`] - oder gar nicht, weil er unter
//! `cargo test` die Vorgabe ist.

use crate::treiber::{Auftrag, Stimme, Treiber};

/// Was an eine Stimme geschickt wurde, ohne die Abtastwerte.
#[derive(Debug, Clone, PartialEq)]
pub struct Eintrag {
    /// Wohin es ging.
    pub stimme: Stimme,
    /// Wie viele Rahmen das Stueck hat.
    pub rahmen: usize,
    /// Wie oft es laufen soll; `-1` heisst endlos.
    pub durchlaeufe: i32,
    /// Ab welcher Stelle, in Millisekunden.
    pub ab_ms: u64,
    /// Nach so vielen Millisekunden ist Schluss.
    pub hoechstdauer_ms: Option<u64>,
    /// Wie lange eingeblendet wird.
    pub einblenden_ms: u64,
}

/// Ein Treiber, der keiner ist.
#[derive(Debug, Default)]
pub struct StummerTreiber {
    auftraege: Vec<Eintrag>,
    gestoppt: usize,
}

impl StummerTreiber {
    /// Ein frisches, stilles Stueck.
    pub fn neu() -> StummerTreiber {
        StummerTreiber::default()
    }

    /// Alles, was seit dem Anlauf abgeschickt wurde.
    pub fn auftraege(&self) -> &[Eintrag] {
        &self.auftraege
    }

    /// Wie oft abgebrochen wurde.
    pub fn gestoppt(&self) -> usize {
        self.gestoppt
    }
}

impl Treiber for StummerTreiber {
    /// Gemessen an pygame unter `SDL_AUDIODRIVER=dummy`: der Treiber
    /// heisst dort ebenfalls `dummy`.
    fn name(&self) -> &str {
        "dummy"
    }

    fn spielen(&mut self, stimme: Stimme, auftrag: Auftrag<'_>) {
        self.auftraege.push(Eintrag {
            stimme,
            rahmen: if auftrag.spuren == 0 {
                0
            } else {
                auftrag.werte.len() / auftrag.spuren as usize
            },
            durchlaeufe: auftrag.durchlaeufe,
            ab_ms: auftrag.ab_ms,
            hoechstdauer_ms: auftrag.hoechstdauer_ms,
            einblenden_ms: auftrag.einblenden_ms,
        });
    }

    fn stoppen(&mut self, _stimme: Stimme) {
        self.gestoppt += 1;
    }

    fn pausieren(&mut self, _stimme: Stimme) {}

    fn fortsetzen(&mut self, _stimme: Stimme) {}

    fn lautstaerke(&mut self, _stimme: Stimme, _wert: f32) {}

    fn kanaele_setzen(&mut self, _anzahl: usize) {}

    fn ist_stumm(&self) -> bool {
        true
    }
}
