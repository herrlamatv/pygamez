//! zustand.rs
//! ==========
//! Das Uhrwerk. Hier liegt alles, was man von aussen sehen kann - und
//! hier wird es aus der Zeit gerechnet, nicht aus dem Treiber gelesen.
//!
//! ## Warum am Strang und nicht prozessweit
//!
//! pygame haelt seinen Mixerzustand prozessweit. Hier liegt er
//! **thread-lokal**, aus denselben zwei Gruenden wie bei
//! `rustgame-display`: rodios `OutputStream` gehoert dem Strang, der ihn
//! geoeffnet hat und ist nicht `Send`, und so stoeren sich die Tests
//! nicht, die `cargo test` nebenlaeufig faehrt.
//!
//! ## Das Nachfuehren
//!
//! Es gibt **keinen Zeitstrang**. Stattdessen fuehrt [`mit`] den Zustand
//! bei jedem Zugriff nach: wer `get_busy()` fragt, laesst damit zuerst
//! alles ablaufen, was seit dem letzten Mal faellig geworden ist - Toene
//! enden, Endereignisse entstehen, Warteschlangen ruecken auf,
//! Ausblendungen werden leiser gestellt.
//!
//! Das hat einen angenehmen Nebeneffekt: die Buchfuehrung ist von der
//! Tonausgabe vollstaendig unabhaengig. Deshalb liefert der stille
//! Treiber dieselben Zahlen wie der hoerbare, und deshalb braucht kein
//! Test ein Tongeraet.

use std::cell::RefCell;

use rustgame_core::time::get_ticks;

use crate::ereignis::Endereignis;
use crate::konstanten::*;
use crate::mixer::Einstellungen;
use crate::sound::Sound;
use crate::treiber::{Auftrag, Stimme, Treiber, Wahl};

// ---------------------------------------------------------------------------
//  Ein laufendes Stueck
// ---------------------------------------------------------------------------

/// Was an einer Stimme gerade laeuft.
///
/// Alle Zeiten sind **verstrichene** Millisekunden, also ohne die
/// Pausen - so rechnen Grenze, Ausblendung und `get_pos` alle mit
/// derselben Groesse, und Pausieren ist ein Einzeiler.
#[derive(Debug, Clone)]
pub(crate) struct Lauf {
    /// `get_ticks()` beim Anfangen.
    pub start: u64,
    /// Seit wann pausiert wird, falls pausiert wird.
    pub pause_seit: Option<u64>,
    /// Wie viel Pause schon zusammengekommen ist.
    pub pause_summe: u64,
    /// Rahmen je Durchlauf.
    pub rahmen: u64,
    /// Abtastrate der Rahmen.
    pub frequenz: u32,
    /// Wie oft gelaufen wird; `-1` heisst endlos.
    pub durchlaeufe: i32,
    /// Wo im Stueck begonnen wurde, in Millisekunden.
    pub ab_ms: u64,
    /// Harte Schranke aus `maxtime`.
    pub maxtime: Option<u64>,
    /// Ausblenden: (verstrichene Zeit beim Anfangen, Dauer).
    pub ausblenden: Option<(u64, u64)>,
    /// Was `music.get_pos()` schon vor diesem Abschnitt gezaehlt hat.
    ///
    /// Gemessen: `music.rewind()` und `music.set_pos()` setzen den
    /// Zaehler **nicht** zurueck - er laeuft ueber den Sprung hinweg
    /// weiter. Genau dafuer ist dieses Feld da.
    pub pos_versatz: u64,
}

impl Lauf {
    /// Wie lange seit dem Anfangen wirklich gespielt wurde.
    pub fn verstrichen(&self, jetzt: u64) -> u64 {
        let bis = self.pause_seit.unwrap_or(jetzt);
        bis.saturating_sub(self.start).saturating_sub(self.pause_summe)
    }

    /// Bei welcher verstrichenen Zeit Schluss ist - `None` heisst nie.
    pub fn grenze(&self) -> Option<u64> {
        let mut g = if self.durchlaeufe < 0 || self.frequenz == 0 {
            None
        } else {
            Some(
                (self.rahmen * self.durchlaeufe as u64 * 1000 / self.frequenz as u64)
                    .saturating_sub(self.ab_ms),
            )
        };
        if let Some(mt) = self.maxtime {
            g = Some(g.map_or(mt, |x| x.min(mt)));
        }
        if let Some((ab, dauer)) = self.ausblenden {
            let f = ab.saturating_add(dauer);
            g = Some(g.map_or(f, |x| x.min(f)));
        }
        g
    }

    /// Ist es vorbei?
    pub fn fertig(&self, jetzt: u64) -> bool {
        match self.grenze() {
            Some(g) => self.verstrichen(jetzt) >= g,
            None => false,
        }
    }

    /// Wie laut es gerade sein soll, als Faktor auf die eingestellte
    /// Lautstaerke - waehrend einer Ausblendung faellt das von 1 auf 0.
    pub fn ausblendfaktor(&self, jetzt: u64) -> f32 {
        match self.ausblenden {
            Some((ab, dauer)) if dauer > 0 => {
                let seit = self.verstrichen(jetzt).saturating_sub(ab);
                1.0 - (seit as f32 / dauer as f32).clamp(0.0, 1.0)
            }
            Some(_) => 0.0,
            None => 1.0,
        }
    }
}

// ---------------------------------------------------------------------------
//  Kanal und Musik
// ---------------------------------------------------------------------------

/// Ein Mischkanal, so wie pygame ihn fuehrt.
#[derive(Debug, Default)]
pub(crate) struct Kanal {
    /// Was gerade gespielt wird.
    pub sound: Option<Sound>,
    /// Was danach drankommt.
    pub schlange: Option<Sound>,
    /// Welches Ereignis das Ende meldet; `0` heisst keines.
    pub endevent: u32,
    /// Lautstaerke in SDLs Stufen, 0 bis 128.
    pub lautstaerke: i32,
    /// Balance links/rechts, je 0 bis 255.
    ///
    /// Gemessen: **nicht auslesbar.** `Channel.get_volume()` liefert nur
    /// die Lautstaerke, und `set_volume(l, r)` setzt die auf 128 zurueck.
    /// Der Wert wird hier gefuehrt, damit
    /// [`crate::Channel::set_volume_stereo`] und
    /// [`crate::Channel::set_source_location`] sich gegenseitig
    /// ueberschreiben wie in SDL_mixer, wo beide dieselben
    /// `position_args` beschreiben. In den Treiber geht er **nicht**:
    /// [`crate::treiber::Treiber::lautstaerke`] hat einen einzigen
    /// Regler, keine zwei Seiten.
    pub panning: (i32, i32),
    /// Entfernung aus `Mix_SetPosition`, in SDLs eigener Zaehlweise.
    ///
    /// **Umgedreht zu `Channel.set_source_location(angle, distance)`:**
    /// `Mix_SetPosition` rechnet `distance = 255 - distance` um, bevor es
    /// den Wert ablegt (`effect_position.c`, `Mix_SetPosition`). Hier
    /// steht die umgerechnete Zahl, also `255` fuer "ganz nah, kein
    /// Abzug" und `0` fuer "ganz weit weg, still".
    pub entfernung: i32,
    /// Was gerade laeuft.
    pub lauf: Option<Lauf>,
}

impl Kanal {
    fn neu() -> Kanal {
        Kanal {
            lautstaerke: MAX_VOLUME,
            panning: (MAX_PANNING, MAX_PANNING),
            entfernung: MAX_PANNING,
            ..Default::default()
        }
    }

    /// Die Lautstaerke als Faktor, mit Ausblendung und Entfernung.
    pub fn faktor(&self, jetzt: u64) -> f32 {
        let grund = self.lautstaerke as f32 / MAX_VOLUME as f32;
        let ton = self
            .sound
            .as_ref()
            .map_or(1.0, |s| s.get_volume() as f32)
            .clamp(0.0, 1.0);
        let aus = self.lauf.as_ref().map_or(1.0, |l| l.ausblendfaktor(jetzt));
        // `distance_f = ((float) distance) / 255.0f` - uebertragen aus
        // `effect_position.c`, wo der Positionseffekt jeden Abtastwert
        // damit multipliziert. Der Winkel bleibt aussen vor; er teilt
        // sich in SDL auf links und rechts auf, und dafuer fehlt dem
        // Treiber die zweite Seite.
        let fern = self.entfernung as f32 / MAX_PANNING as f32;
        grund * ton * aus * fern
    }
}

/// Die Musikspur.
#[derive(Debug, Default)]
pub(crate) struct Musik {
    /// Das geladene Stueck.
    pub geladen: Option<Sound>,
    /// Was an Beschriftung in der Datei stand - `music.get_metadata()`.
    ///
    /// Haengt an der **Datei**, nicht am Klang: die Abtastwerte wissen
    /// nichts von Titel und Album, und ein selbst erzeugtes Stueck hat
    /// beides nicht. Gefuellt wird es beim Laden, geleert beim Entladen.
    pub etiketten: crate::id3::Etiketten,
    /// Was danach drankommt, mit seiner Wiederholungszahl.
    pub schlange: Option<(Sound, i32)>,
    /// Was gerade laeuft.
    pub lauf: Option<Lauf>,
    /// Lautstaerke in SDLs Stufen, 0 bis 128.
    pub lautstaerke: i32,
    /// Welches Ereignis das Ende meldet.
    pub endevent: u32,
}

// ---------------------------------------------------------------------------
//  Der Zustand selbst
// ---------------------------------------------------------------------------

/// Was `get_init()` meldet.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Spec {
    /// Abtastrate in Hertz.
    pub frequenz: i32,
    /// Bits je Wert, negativ heisst vorzeichenbehaftet.
    pub groesse: i32,
    /// Ausgabespuren: 1, 2, 4 oder 6.
    pub spuren: i32,
    /// Puffergroesse in Rahmen.
    ///
    /// `get_init()` verraet sie **nicht** - gemessen liefert es nur drei
    /// Werte. Sie steht hier, weil `init()` sie auf eine Zweierpotenz
    /// aufrundet und das sonst niemand nachvollziehen koennte.
    pub puffer: i32,
}

/// Der gesamte Mixerzustand eines Strangs.
pub(crate) struct Zustand {
    /// Womit der Mixer laeuft; `None` heisst: nicht angelaufen.
    pub offen: Option<Spec>,
    /// Was `pre_init` gewuenscht hat.
    pub wunsch: Einstellungen,
    /// Welcher Treiber genommen werden soll.
    pub wahl: Wahl,
    /// Der Treiber, solange der Mixer laeuft.
    pub treiber: Option<Box<dyn Treiber>>,
    /// Die Mischkanaele.
    ///
    /// Diese Liste **schrumpft nie**, auch wenn `set_num_channels`
    /// verkleinert - genau wie pygames `channeldata`, das nur waechst.
    /// Wie viele davon gelten, steht in [`Zustand::anzahl`]. So ueberlebt
    /// ein gesetztes `endevent` ein Verkleinern und wieder Vergroessern.
    pub kanaele: Vec<Kanal>,
    /// Wie viele Kanaele gerade gelten - `get_num_channels()`.
    pub anzahl: usize,
    /// Wie viele Kanaele fuer `Channel(n)` freigehalten sind.
    pub reserviert: usize,
    /// Die Musikspur.
    pub musik: Musik,
    /// Faellige Endereignisse, die noch niemand abgeholt hat.
    pub ereignisse: Vec<Endereignis>,
    /// Der Pfad zur MIDI-Klangbank - gemerkt, aber wirkungslos, weil
    /// rustgame kein MIDI spielt. Siehe `mixer::set_soundfont`.
    pub soundfont: Option<String>,
}

impl Default for Zustand {
    fn default() -> Zustand {
        Zustand {
            offen: None,
            wunsch: Einstellungen::default(),
            wahl: Wahl::Automatisch,
            treiber: None,
            kanaele: Vec::new(),
            anzahl: 0,
            reserviert: 0,
            musik: Musik::default(),
            ereignisse: Vec::new(),
            soundfont: None,
        }
    }
}

thread_local! {
    static ZUSTAND: RefCell<Zustand> = RefCell::new(Zustand::default());
}

/// Mehr Endereignisse als das hebt niemand auf.
///
/// pygames Schlange fasst 128; was darueber hinaus entsteht, faellt
/// weg. Ohne diese Schranke wuerde ein Endlosklang mit gesetztem
/// `endevent`, den niemand abfragt, den Speicher vollschreiben.
const MAX_EREIGNISSE: usize = 128;

/// Ruft `f` mit dem Zustand - und fuehrt ihn vorher nach.
///
/// **Jeder** oeffentliche Zugriff dieser Kiste geht hier durch. Das ist
/// die Stelle, an der die Zeit vergeht.
pub(crate) fn mit<R>(f: impl FnOnce(&mut Zustand) -> R) -> R {
    let (ergebnis, abzugeben) = ZUSTAND.with(|z| {
        let mut z = z.borrow_mut();
        nachfuehren(&mut z);
        let r = f(&mut z);
        // Die Ereignisse verlassen die Ausleihe, bevor die Senke sie
        // sieht - sonst koennte eine Senke, die zurueck in den Mixer
        // greift, die uebliche RefCell-Panik ausloesen.
        let e = if crate::ereignis::senke_gesetzt() {
            std::mem::take(&mut z.ereignisse)
        } else {
            Vec::new()
        };
        (r, e)
    });
    for e in abzugeben {
        crate::ereignis::an_senke(e);
    }
    ergebnis
}

/// Setzt alles zurueck - fuer `mixer.quit()`.
pub(crate) fn zuruecksetzen(z: &mut Zustand) {
    z.offen = None;
    z.treiber = None;
    z.kanaele.clear();
    z.anzahl = 0;
    z.reserviert = 0;
    z.musik = Musik::default();
}

impl Zustand {
    /// Klemmt, wenn der Mixer nicht laeuft.
    pub fn pruefen(&self) -> crate::Ergebnis<Spec> {
        self.offen.ok_or(crate::Fehler::NichtInitialisiert)
    }

    /// Legt die Kanalzahl auf `anzahl` fest.
    ///
    /// Gemessen: wird verkleinert, **verstummen** die wegfallenden
    /// Kanaele sofort - `set_num_channels(2)` bei drei laufenden Toenen
    /// laesst zwei uebrig. Ihr Endereignis behalten sie trotzdem, denn
    /// die Liste selbst bleibt stehen.
    pub fn kanaele_setzen(&mut self, anzahl: usize) {
        while self.kanaele.len() < anzahl {
            self.kanaele.push(Kanal::neu());
        }
        for i in anzahl..self.anzahl {
            self.kanaele[i].sound = None;
            self.kanaele[i].schlange = None;
            self.kanaele[i].lauf = None;
            if let Some(t) = self.treiber.as_mut() {
                t.stoppen(Stimme::Kanal(i));
            }
        }
        self.anzahl = anzahl;
        self.reserviert = self.reserviert.min(anzahl);
        if let Some(t) = self.treiber.as_mut() {
            t.kanaele_setzen(anzahl);
        }
    }

    /// Schickt einem Kanal den Auftrag, etwas zu spielen.
    pub fn beauftragen(&mut self, i: usize, ab_ms: u64, einblenden_ms: u64) {
        let jetzt = get_ticks();
        let (Some(sound), Some((durchlaeufe, maxtime))) = (
            self.kanaele[i].sound.clone(),
            self.kanaele[i]
                .lauf
                .as_ref()
                .map(|l| (l.durchlaeufe, l.maxtime)),
        ) else {
            return;
        };
        let faktor = self.kanaele[i].faktor(jetzt);
        // Einmal umrechnen, dann leihen: der Treiber kopiert die Werte
        // ohnehin in seinen eigenen Puffer.
        let werte = sound.werte();
        if let Some(t) = self.treiber.as_mut() {
            t.spielen(
                Stimme::Kanal(i),
                Auftrag {
                    werte: &werte,
                    spuren: sound.spuren(),
                    frequenz: sound.frequenz(),
                    durchlaeufe,
                    ab_ms,
                    hoechstdauer_ms: maxtime,
                    einblenden_ms,
                    lautstaerke: faktor,
                },
            );
        }
    }

    /// Schickt der Musikspur den Auftrag, etwas zu spielen.
    pub fn musik_beauftragen(&mut self, ab_ms: u64, einblenden_ms: u64) {
        let (Some(stueck), Some(durchlaeufe)) = (
            self.musik.geladen.clone(),
            self.musik.lauf.as_ref().map(|l| l.durchlaeufe),
        ) else {
            return;
        };
        let faktor = self.musik.lautstaerke as f32 / MAX_VOLUME as f32;
        let werte = stueck.werte();
        if let Some(t) = self.treiber.as_mut() {
            t.spielen(
                Stimme::Musik,
                Auftrag {
                    werte: &werte,
                    spuren: stueck.spuren(),
                    frequenz: stueck.frequenz(),
                    durchlaeufe,
                    ab_ms,
                    hoechstdauer_ms: None,
                    einblenden_ms,
                    lautstaerke: faktor,
                },
            );
        }
    }

    /// Legt ein Endereignis ab, sofern eines gesetzt ist.
    fn melden(&mut self, typ: u32, code: Option<usize>) {
        if typ == NOEVENT {
            return;
        }
        if self.ereignisse.len() < MAX_EREIGNISSE {
            self.ereignisse.push(Endereignis { typ, code });
        }
    }

    /// Beendet einen Kanal so, wie pygame es tut - mit allem, was
    /// daranhaengt.
    ///
    /// Gemessen und ueberraschend: `Channel.stop()` ist **nicht** nur ein
    /// Abbruch. SDL_mixer ruft auch beim Abbrechen den Fertig-Rueckruf,
    /// und pygames Rueckruf tut zwei Dinge - er schickt das Endereignis,
    /// und er laesst die Warteschlange aufruecken. Nach
    ///
    /// ```text
    /// c.play(lang); c.queue(kurz); c.stop()
    /// ```
    ///
    /// spielt in echtem pygame also `kurz` - `get_busy()` ist `True`. Das
    /// steht so in keiner Doku; gemessen ist es eindeutig, und darum
    /// macht diese Kiste es genauso.
    pub fn kanal_beenden(&mut self, i: usize, ab_tick: u64) {
        if self.kanaele[i].lauf.is_none() {
            return;
        }
        let typ = self.kanaele[i].endevent;
        self.melden(typ, Some(i));

        match self.kanaele[i].schlange.take() {
            Some(naechster) => {
                let lauf = Lauf {
                    start: ab_tick,
                    pause_seit: None,
                    pause_summe: 0,
                    rahmen: naechster.rahmen() as u64,
                    frequenz: naechster.frequenz(),
                    durchlaeufe: 1,
                    ab_ms: 0,
                    maxtime: None,
                    ausblenden: None,
                    pos_versatz: 0,
                };
                self.kanaele[i].sound = Some(naechster);
                self.kanaele[i].lauf = Some(lauf);
                self.beauftragen(i, 0, 0);
            }
            None => {
                self.kanaele[i].sound = None;
                self.kanaele[i].lauf = None;
                if let Some(t) = self.treiber.as_mut() {
                    t.stoppen(Stimme::Kanal(i));
                }
            }
        }
    }

    /// Beendet die Musik.
    ///
    /// Anders als bei den Kanaelen rueckt hier **nichts** auf: pygames
    /// `music.stop()` und `music.fadeout()` werfen die Warteschlange
    /// ausdruecklich vorher weg ("To prevent the queue_music from
    /// playing"). Gemessen stimmt das - `get_busy()` ist danach `False`.
    /// Nur das Endereignis kommt, auch beim Abbrechen.
    pub fn musik_beenden(&mut self, mit_schlange: bool, ab_tick: u64) {
        if self.musik.lauf.is_none() {
            return;
        }
        let typ = self.musik.endevent;
        self.melden(typ, None);

        let naechstes = if mit_schlange {
            self.musik.schlange.take()
        } else {
            self.musik.schlange = None;
            None
        };
        match naechstes {
            Some((stueck, durchlaeufe)) => {
                let alt_pos = self
                    .musik
                    .lauf
                    .as_ref()
                    .map_or(0, |l| l.pos_versatz + l.verstrichen(ab_tick));
                self.musik.lauf = Some(Lauf {
                    start: ab_tick,
                    pause_seit: None,
                    pause_summe: 0,
                    rahmen: stueck.rahmen() as u64,
                    frequenz: stueck.frequenz(),
                    durchlaeufe,
                    ab_ms: 0,
                    maxtime: None,
                    ausblenden: None,
                    pos_versatz: alt_pos,
                });
                self.musik.geladen = Some(stueck);
                self.musik_beauftragen(0, 0);
            }
            None => {
                self.musik.lauf = None;
                if let Some(t) = self.treiber.as_mut() {
                    t.stoppen(Stimme::Musik);
                }
            }
        }
    }
}

// ---------------------------------------------------------------------------
//  Nachfuehren
// ---------------------------------------------------------------------------

/// Laesst die Zeit vergehen: beendet, was faellig ist, und stellt
/// Ausblendungen leiser.
pub(crate) fn nachfuehren(z: &mut Zustand) {
    if z.offen.is_none() {
        return;
    }
    let jetzt = get_ticks();

    for i in 0..z.anzahl {
        // Ein Kanal kann mehrfach hintereinander faellig werden, wenn die
        // Warteschlange einen sehr kurzen Klang enthaelt. Die Schleife
        // haelt an, sobald nichts mehr laeuft oder noch etwas laeuft.
        while let Some(lauf) = z.kanaele[i].lauf.as_ref() {
            if !lauf.fertig(jetzt) {
                // Waehrend einer Ausblendung leiser stellen - so wird sie
                // auch wirklich hoerbar.
                if lauf.ausblenden.is_some() {
                    let f = z.kanaele[i].faktor(jetzt);
                    if let Some(t) = z.treiber.as_mut() {
                        t.lautstaerke(Stimme::Kanal(i), f);
                    }
                }
                break;
            }
            // Die genaue Zeit, zu der es zu Ende war - so schleppt eine
            // lange nicht abgefragte Warteschlange keinen Versatz mit.
            let ende = lauf
                .start
                .saturating_add(lauf.pause_summe)
                .saturating_add(lauf.grenze().unwrap_or(0))
                .min(jetzt);
            z.kanal_beenden(i, ende);
        }
    }

    while let Some(lauf) = z.musik.lauf.as_ref() {
        if !lauf.fertig(jetzt) {
            if lauf.ausblenden.is_some() {
                let f = (z.musik.lautstaerke as f32 / MAX_VOLUME as f32)
                    * lauf.ausblendfaktor(jetzt);
                if let Some(t) = z.treiber.as_mut() {
                    t.lautstaerke(Stimme::Musik, f);
                }
            }
            break;
        }
        let ende = lauf
            .start
            .saturating_add(lauf.pause_summe)
            .saturating_add(lauf.grenze().unwrap_or(0))
            .min(jetzt);
        z.musik_beenden(true, ende);
    }
}
