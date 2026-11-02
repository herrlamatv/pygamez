//! midi.rs
//! =======
//! `pygame.midi` - die **Geraeteseite**, ueber die Kiste `midir`.
//!
//! ## Warum es das jetzt gibt
//!
//! Hier stand nichts, und im Modul daneben stand die Begruendung:
//! `Input` und `Output` braeuchten PortMidi, und PortMidi gaebe es nur
//! ueber FFI. Der erste Teil ist wahr, der zweite ist eine
//! Verwechslung - dieselbe, die bei `system.get_total_ram` schon einmal
//! aufgeloest wurde: **`unsafe` braucht irgendwer, aber nicht dieses
//! Programm.** `midir` spricht unter Windows mit WinMM, unter Linux mit
//! ALSA und unter macOS mit CoreMIDI, und bietet nach aussen eine
//! sichere Schnittstelle.
//!
//! Die Probe, die es ausgeloest hat, ist eine Zeile:
//!
//! ```text
//! pygame.midi.get_count()  ->  2
//!   (b'MMSystem', b'Microsoft MIDI Mapper',        0, 1, 0)
//!   (b'MMSystem', b'Microsoft GS Wavetable Synth', 0, 1, 0)
//! ```
//!
//! Auf dieser Maschine **gibt es** MIDI-Ausgaenge, und pygame benutzt
//! sie. Wer dieselben Klassen anbietet und bei jedem Aufruf einen Fehler
//! wirft, hat die Namen und nicht die Sache.
//!
//! ## Was gemessen ist
//!
//! | Frage | pygame hier |
//! |---|---|
//! | `get_count()` | `2` |
//! | `get_default_input_id()` | `-1` (es gibt keinen Eingang) |
//! | `get_default_output_id()` | `0` |
//! | `get_device_info(0)` | `(b'MMSystem', b'...', 0, 1, 0)` |
//! | `get_device_info(2)` | `None` - **kein Fehler** |
//!
//! Die letzte Zeile ist die, auf die man nicht kommt: eine Nummer
//! ausserhalb des Bereichs gibt `None` und nicht `IndexError`. pygames
//! eigener Test prueft es ausdruecklich.
//!
//! ## Die Reihenfolge der Geraete
//!
//! PortMidi fuehrt **eine** Liste, in der jedes Geraet einmal steht und
//! sich als Eingang oder Ausgang bezeichnet. `midir` fuehrt zwei. Hier
//! werden sie in der Reihenfolge zusammengelegt, in der `pm_winmm_init`
//! (`pmwinmm.c`) sie eintraegt:
//!
//! ```c
//! pm_winmm_mapper_input();    /* midiInGetDevCaps(MIDIMAPPER, ..)  */
//! pm_winmm_mapper_output();   /* midiOutGetDevCaps(MIDIMAPPER, ..) */
//! pm_winmm_general_inputs();  /* 0 .. midiInGetNumDevs()           */
//! pm_winmm_general_outputs(); /* 0 .. midiOutGetNumDevs()          */
//! ```
//!
//! ## Der `Microsoft MIDI Mapper`
//!
//! Hier stand "erst die Eingaenge, dann die Ausgaenge", und der Mapper
//! fehlte: er ist kein nummeriertes Geraet, sondern `MIDI_MAPPER`
//! (`(UINT)-1`), und `midir` zaehlt nur `0 .. midiOutGetNumDevs()`. pygame
//! hatte darum zwei Geraete, rustgame eins, und **Nummer 0 war ein
//! anderes**: bei pygame der Mapper, hier der Wavetable-Synthesizer. Die
//! Probe `midi.Geraeteliste` liess die Anzahl weg und nannte das eine
//! benannte Grenze - in der Grenzentafel stand aber keine.
//!
//! Den Mapper-**Eingang** gibt es unter Windows nicht (`midiInGetDevCaps`
//! mit `MIDIMAPPER` scheitert, PortMidi traegt dann nichts ein). Den
//! Mapper-**Ausgang** gibt es, und `midimap.dll` leitet ihn auf das
//! Geraet, das `HKCU\Software\Microsoft\Windows\CurrentVersion\Multimedia\
//! MIDIMap` unter `szPname` nennt - fehlt der Eintrag, auf das erste. So
//! oeffnet ihn auch [`Ausgang::oeffnen`]. Der Name ist der, den WinMM fuer
//! `MIDI_MAPPER` meldet; gemessen auf einem deutschen Windows 11 steht
//! dort das englische `Microsoft MIDI Mapper`.
//!
//! **Nicht gemessen** ist ein Rechner ganz ohne Ausgabegeraet. Dort fuehrt
//! der Mapper nirgends hin; er wird hier dann nicht aufgefuehrt.

use std::collections::VecDeque;
use std::sync::{Arc, Mutex};

use midir::{MidiInput, MidiInputConnection, MidiOutput, MidiOutputConnection};

use crate::{Ergebnis, Fehler};

/// Wie sich ein Geraet nach aussen beschreibt.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Geraet {
    /// Der Name der Schnittstelle - unter Windows `MMSystem`.
    pub schnittstelle: String,
    /// Der Name des Geraets, wie das System ihn nennt.
    pub name: String,
    /// Kann man davon lesen?
    pub eingang: bool,
    /// Kann man darauf schreiben?
    pub ausgang: bool,
}

/// Der Name der Schnittstelle, wie PortMidi sie nennt.
///
/// PortMidi schreibt dort den Namen der Systemschnittstelle, nicht den
/// der Kiste: unter Windows `MMSystem`, unter Linux `ALSA`, unter macOS
/// `CoreMIDI`. Gemessen an pygame auf dieser Maschine - dort steht
/// `b'MMSystem'`.
fn schnittstelle() -> &'static str {
    if cfg!(target_os = "windows") {
        "MMSystem"
    } else if cfg!(target_os = "macos") {
        "CoreMIDI"
    } else {
        "ALSA"
    }
}

/// Der Name, den WinMM fuer `MIDI_MAPPER` meldet.
const MAPPER: &str = "Microsoft MIDI Mapper";

/// Wohinter ein Eintrag der Liste steht - damit `oeffnen` den richtigen
/// `midir`-Anschluss findet, ohne die Nummern nachzurechnen.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Herkunft {
    /// `MIDI_MAPPER` - fuehrt auf einen der Ausgaenge, siehe [`mapper_ziel`].
    Mapper,
    /// Der n-te Eingang von `midir`.
    Eingang(usize),
    /// Der n-te Ausgang von `midir`.
    Ausgang(usize),
}

/// Die Liste in PortMidis Reihenfolge - siehe "Die Reihenfolge der
/// Geraete" oben.
fn liste() -> Vec<(Geraet, Herkunft)> {
    let neu = |name: String, eingang: bool| Geraet {
        schnittstelle: schnittstelle().to_string(),
        name,
        eingang,
        ausgang: !eingang,
    };
    let eingaenge: Vec<String> = MidiInput::new("rustgame")
        .map(|e| e.ports().iter().map(|p| e.port_name(p).unwrap_or_default()).collect())
        .unwrap_or_default();
    let ausgaenge: Vec<String> = MidiOutput::new("rustgame")
        .map(|a| a.ports().iter().map(|p| a.port_name(p).unwrap_or_default()).collect())
        .unwrap_or_default();
    let mut aus = Vec::new();
    if cfg!(windows) && !ausgaenge.is_empty() {
        aus.push((neu(MAPPER.to_string(), false), Herkunft::Mapper));
    }
    for (i, n) in eingaenge.into_iter().enumerate() {
        aus.push((neu(n, true), Herkunft::Eingang(i)));
    }
    for (i, n) in ausgaenge.into_iter().enumerate() {
        aus.push((neu(n, false), Herkunft::Ausgang(i)));
    }
    aus
}

/// Welchen Ausgang `midimap.dll` fuer den Mapper nimmt: den, den
/// `MIDIMap\szPname` nennt, sonst den ersten.
fn mapper_ziel(namen: &[String]) -> usize {
    #[cfg(windows)]
    {
        let gewaehlt = windows_registry::CURRENT_USER
            .open(r"Software\Microsoft\Windows\CurrentVersion\Multimedia\MIDIMap")
            .and_then(|k| k.get_string("szPname"));
        if let Ok(n) = gewaehlt {
            if let Some(i) = namen.iter().position(|x| *x == n) {
                return i;
            }
        }
    }
    let _ = namen;
    0
}

/// Alle Geraete in PortMidis Reihenfolge.
pub fn geraete() -> Vec<Geraet> {
    liste().into_iter().map(|(g, _)| g).collect()
}

/// Wie viele Geraete es gibt.
pub fn anzahl() -> usize {
    geraete().len()
}

/// Die Nummer des ersten Eingangs - oder `-1`.
pub fn erster_eingang() -> i32 {
    geraete()
        .iter()
        .position(|g| g.eingang)
        .map(|i| i as i32)
        .unwrap_or(-1)
}

/// Die Nummer des ersten Ausgangs - oder `-1`.
pub fn erster_ausgang() -> i32 {
    geraete()
        .iter()
        .position(|g| g.ausgang)
        .map(|i| i as i32)
        .unwrap_or(-1)
}

/// Ein Geraet, oder `None` bei einer Nummer ausserhalb des Bereichs.
pub fn geraet(nummer: usize) -> Option<Geraet> {
    geraete().into_iter().nth(nummer)
}

// ---------------------------------------------------------------------------
//  Ausgang
// ---------------------------------------------------------------------------

/// Ein offener MIDI-Ausgang - `pygame.midi.Output`.
pub struct Ausgang {
    draht: Option<MidiOutputConnection>,
}

impl Ausgang {
    /// Oeffnet den Ausgang mit dieser Nummer.
    pub fn oeffnen(nummer: usize) -> Ergebnis<Ausgang> {
        let Some((g, herkunft)) = liste().into_iter().nth(nummer) else {
            return Err(Fehler::NichtUnterstuetzt("Device id invalid, out of range.".into()));
        };
        if !g.ausgang {
            return Err(Fehler::NichtUnterstuetzt("Device id is not an output device.".into()));
        }
        let a = MidiOutput::new("rustgame")
            .map_err(|_| Fehler::NichtUnterstuetzt("MIDI could not be opened".into()))?;
        let ports = a.ports();
        let i = match herkunft {
            Herkunft::Ausgang(i) => i,
            Herkunft::Mapper => {
                let namen: Vec<String> =
                    ports.iter().map(|p| a.port_name(p).unwrap_or_default()).collect();
                mapper_ziel(&namen)
            }
            Herkunft::Eingang(_) => {
                return Err(Fehler::NichtUnterstuetzt(
                    "Device id is not an output device.".into(),
                ))
            }
        };
        let Some(p) = ports.get(i) else {
            return Err(Fehler::NichtUnterstuetzt("Device id invalid, out of range.".into()));
        };
        let draht = a
            .connect(p, "rustgame-out")
            .map_err(|_| Fehler::NichtUnterstuetzt("MIDI output could not be opened".into()))?;
        Ok(Ausgang { draht: Some(draht) })
    }

    /// Schickt rohe Bytes.
    ///
    /// **Jede Nachricht einzeln.** `midir` erwartet eine vollstaendige
    /// MIDI-Nachricht je Aufruf; wer zwei hintereinander in einen Puffer
    /// legt, bekommt sie nicht gespielt. `pygame.midi.Output.write`
    /// nimmt dagegen eine Liste, und die wird darum hier aufgeteilt.
    pub fn senden(&mut self, bytes: &[u8]) -> Ergebnis<()> {
        let Some(d) = self.draht.as_mut() else {
            return Err(Fehler::NichtUnterstuetzt("Output device is closed.".into()));
        };
        d.send(bytes)
            .map_err(|_| Fehler::NichtUnterstuetzt("MIDI message could not be sent".into()))
    }

    /// Schliesst den Ausgang. Ein zweites Mal ist kein Fehler.
    pub fn schliessen(&mut self) {
        if let Some(d) = self.draht.take() {
            d.close();
        }
    }

    /// Ist er noch offen?
    pub fn offen(&self) -> bool {
        self.draht.is_some()
    }
}

// ---------------------------------------------------------------------------
//  Eingang
// ---------------------------------------------------------------------------

/// Was an einem Eingang angekommen ist.
type Posteingang = Arc<Mutex<VecDeque<(Vec<u8>, u64)>>>;

/// Ein offener MIDI-Eingang - `pygame.midi.Input`.
///
/// `midir` ruft bei jeder Nachricht einen Rueckruf; pygames `read` fragt
/// dagegen ab. Zwischen beiden steht hier eine Schlange: der Rueckruf
/// legt hinein, `lesen` nimmt heraus.
pub struct Eingang {
    draht: Option<MidiInputConnection<Posteingang>>,
    post: Posteingang,
    /// Wie viele Nachrichten die Schlange hoechstens haelt.
    ///
    /// pygames `buffer_size` ist die Groesse des PortMidi-Puffers; laeuft
    /// er ueber, gehen die **aeltesten** Nachrichten verloren. Dasselbe
    /// hier - eine Schlange ohne Grenze waere ein Leck, das erst nach
    /// Stunden auffaellt.
    platz: usize,
}

impl Eingang {
    /// Oeffnet den Eingang mit dieser Nummer.
    pub fn oeffnen(nummer: usize, platz: usize) -> Ergebnis<Eingang> {
        let Some((g, herkunft)) = liste().into_iter().nth(nummer) else {
            return Err(Fehler::NichtUnterstuetzt("Device id invalid, out of range.".into()));
        };
        let Herkunft::Eingang(i) = herkunft else {
            return Err(Fehler::NichtUnterstuetzt("Device id is not an input device.".into()));
        };
        debug_assert!(g.eingang);
        let mut e = MidiInput::new("rustgame")
            .map_err(|_| Fehler::NichtUnterstuetzt("MIDI could not be opened".into()))?;
        e.ignore(midir::Ignore::None);
        let ports = e.ports();
        // Der i-te Eingang von `midir` - **nicht** `nummer`: vor den
        // Eingaengen steht unter Windows der Mapper.
        let Some(p) = ports.get(i) else {
            return Err(Fehler::NichtUnterstuetzt("Device id invalid, out of range.".into()));
        };
        let post: Posteingang = Arc::new(Mutex::new(VecDeque::new()));
        let grenze = platz.max(1);
        let draht = e
            .connect(
                p,
                "rustgame-in",
                move |zeit, nachricht, ziel: &mut Posteingang| {
                    let mut q = match ziel.lock() {
                        Ok(q) => q,
                        Err(_) => return,
                    };
                    while q.len() >= grenze {
                        q.pop_front();
                    }
                    // `midir` zaehlt in Mikrosekunden, pygame in
                    // Millisekunden - dieselbe Einheit wie
                    // `pygame.midi.time()`.
                    q.push_back((nachricht.to_vec(), zeit / 1000));
                },
                Arc::clone(&post),
            )
            .map_err(|_| Fehler::NichtUnterstuetzt("MIDI input could not be opened".into()))?;
        Ok(Eingang {
            draht: Some(draht),
            post,
            platz: grenze,
        })
    }

    /// Liegt etwas an?
    pub fn anliegt(&self) -> Ergebnis<bool> {
        if self.draht.is_none() {
            return Err(Fehler::NichtUnterstuetzt("Input device is closed.".into()));
        }
        Ok(self.post.lock().map(|q| !q.is_empty()).unwrap_or(false))
    }

    /// Nimmt bis zu `anzahl` Nachrichten heraus.
    pub fn lesen(&self, anzahl: usize) -> Ergebnis<Vec<(Vec<u8>, u64)>> {
        if self.draht.is_none() {
            return Err(Fehler::NichtUnterstuetzt("Input device is closed.".into()));
        }
        let mut q = match self.post.lock() {
            Ok(q) => q,
            Err(_) => return Ok(Vec::new()),
        };
        let n = anzahl.min(q.len()).min(self.platz);
        Ok(q.drain(..n).collect())
    }

    /// Schliesst den Eingang. Ein zweites Mal ist kein Fehler.
    pub fn schliessen(&mut self) {
        if let Some(d) = self.draht.take() {
            d.close();
        }
    }

    /// Ist er noch offen?
    pub fn offen(&self) -> bool {
        self.draht.is_some()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Die Geraeteliste beschreibt sich widerspruchsfrei.
    ///
    /// Geprueft wird **nicht**, dass es Geraete gibt - auf einem
    /// Bauknecht ohne Soundkarte gibt es keine, und das ist kein Fehler.
    /// Geprueft wird, dass jede Nummer zu genau einem Geraet gehoert, dass
    /// kein Geraet beides ist, und dass eine Nummer hinter dem Ende
    /// `None` gibt.
    #[test]
    fn die_geraeteliste_ist_widerspruchsfrei() {
        let liste = geraete();
        assert_eq!(anzahl(), liste.len());
        assert!(geraet(liste.len()).is_none(), "hinter dem Ende: None");
        for (i, g) in liste.iter().enumerate() {
            assert_eq!(geraet(i).as_ref(), Some(g), "Nummer {i}");
            assert!(
                g.eingang != g.ausgang,
                "ein Geraet ist Eingang oder Ausgang, nicht beides"
            );
            assert!(!g.schnittstelle.is_empty(), "Schnittstelle hat einen Namen");
        }
        // PortMidis Reihenfolge: unter Windows der Mapper vorn (sobald es
        // einen Ausgang gibt), dann die Eingaenge, dann die Ausgaenge - und
        // die Vorgabenummern zeigen auf das erste der jeweiligen Sorte.
        let roh = super::liste();
        let ohne_mapper: Vec<_> = roh
            .iter()
            .filter(|(_, h)| *h != Herkunft::Mapper)
            .collect();
        if let Some(a) = ohne_mapper.iter().position(|(g, _)| g.ausgang) {
            assert!(
                ohne_mapper[..a].iter().all(|(g, _)| g.eingang),
                "vor dem ersten echten Ausgang stehen nur Eingaenge"
            );
            if cfg!(windows) {
                assert_eq!(roh[0].1, Herkunft::Mapper, "der Mapper steht vorn");
                assert_eq!(roh[0].0.name, MAPPER);
                assert_eq!(erster_ausgang(), 0, "Vorgabe ist der Mapper");
            }
        } else {
            assert_eq!(erster_ausgang(), -1);
            assert!(roh.iter().all(|(_, h)| *h != Herkunft::Mapper));
        }
        assert_eq!(
            roh.iter().filter(|(_, h)| *h == Herkunft::Mapper).count(),
            usize::from(cfg!(windows) && roh.iter().any(|(g, _)| g.ausgang)),
            "hoechstens ein Mapper"
        );
    }

    /// Ohne `MIDIMap\szPname` - oder mit einem Namen, den es nicht gibt -
    /// fuehrt der Mapper auf den ersten Ausgang.
    #[test]
    fn der_mapper_faellt_auf_den_ersten_ausgang() {
        let namen = vec!["Gibt es nicht 1".to_string(), "Gibt es nicht 2".to_string()];
        assert_eq!(mapper_ziel(&namen), 0);
        assert_eq!(mapper_ziel(&[]), 0);
    }

    /// Eine Nummer ausserhalb des Bereichs gibt einen Fehler, keinen Sturz.
    #[test]
    fn eine_nummer_zu_weit_gibt_einen_fehler() {
        let n = anzahl();
        assert!(Ausgang::oeffnen(n + 10).is_err());
        assert!(Eingang::oeffnen(n + 10, 16).is_err());
    }
}
