//! time.rs
//! =======
//! Gegenstueck zu `pygame.time`: `Clock`, `get_ticks`, `delay`, `wait`.
//!
//! Nur `std::time` - keine Fremdkiste, kein SDL.
//!
//! ## Was hier leicht falsch nachgebaut wird
//!
//! 1. **`Clock::get_fps` mittelt ueber genau zehn Aufrufe**, nicht ueber
//!    alle und nicht gleitend. `clock_tick_base` in `src_c/time.c` zaehlt
//!    mit, und erst wenn zehn `tick` beisammen sind, wird einmal geteilt
//!    und der Zaehler auf null gesetzt. Bis dahin steht `get_fps` auf
//!    `0.0`. Gemessen: bei einer frischen Uhr liefern die ersten zehn
//!    `tick` noch `0.0`, erst der elfte setzt einen Wert.
//!
//! 2. **`get_time` und `get_rawtime` sind nicht dasselbe.** `get_time` ist
//!    die Zeit seit dem vorigen `tick` **einschliesslich** der Wartezeit,
//!    `get_rawtime` die Zeit **ohne** sie - also das, was die Arbeit
//!    wirklich gekostet hat. Nur wenn `tick` ohne Bildrate aufgerufen
//!    wird, sind beide gleich. Wer das verwechselt, baut eine
//!    Auslastungsanzeige, die immer 100% zeigt.
//!
//! 3. **`get_ticks` zaehlt ab `init()`**, nicht ab Programmstart, und
//!    faellt nach [`quit`] wieder auf 0 - genau wie in pygame, wo die Uhr
//!    am Zeitgeber-Teilsystem von SDL haengt. Gemessen: nach
//!    `pygame.quit()` liefert `pygame.time.get_ticks()` glatt `0`, nach
//!    einem erneuten `pygame.init()` faengt es wieder klein an.
//!
//! 4. **`wait` und `delay` sind zwei verschiedene Verfahren.** `wait`
//!    reicht die Wartezeit ans Betriebssystem durch und gibt den Rechner
//!    frei - genau, aber grob. `delay` wartet den Grossteil ebenso und
//!    verbrennt den Rest in einer engen Schleife; das ist genauer und
//!    kostet dafuer eine Kernlast. Beide geben zurueck, wie lange sie
//!    **wirklich** gebraucht haben.

use std::sync::Mutex;
use std::time::{Duration, Instant};

/// Die schlechteste Aufloesung, mit der beim Schlafen zu rechnen ist.
///
/// `WORST_CLOCK_ACCURACY` aus `src_c/time.c`. [`delay`] schlaeft alles bis
/// auf diesen Rest und wartet den Rest aktiv ab.
const SCHLECHTESTE_AUFLOESUNG: u64 = 12;

/// Der Nullpunkt von [`get_ticks`]. `None`, solange nicht angelaufen.
///
/// Ein `Mutex` statt eines `AtomicU64`, weil `Instant` keine Zahl ist. Er
/// wird nur beim Anlauf und beim ersten Zugriff angefasst.
static START: Mutex<Option<Instant>> = Mutex::new(None);

// ---------------------------------------------------------------------------
//  Anlauf und Ende
// ---------------------------------------------------------------------------

/// Setzt die Uhr auf null - Gegenstueck zu dem, was `pygame.init()` fuer
/// das Zeitmodul tut.
///
/// Mehrfaches Aufrufen setzt jedes Mal neu zurueck, genau wie ein
/// `pygame.quit()` mit anschliessendem `pygame.init()`.
pub fn init() {
    if let Ok(mut s) = START.lock() {
        *s = Some(Instant::now());
    }
    zeitgeber_alle_loeschen();
}

/// Haelt die Uhr an - danach liefert [`get_ticks`] wieder `0`.
///
/// Gemessen an pygame: nach `pygame.quit()` gibt `get_ticks()` glatt `0`
/// zurueck, weil SDLs Zeitgeber-Teilsystem nicht mehr laeuft.
pub fn quit() {
    if let Ok(mut s) = START.lock() {
        *s = None;
    }
    zeitgeber_alle_loeschen();
}

/// Laeuft die Uhr gerade?
pub fn get_init() -> bool {
    START.lock().map(|s| s.is_some()).unwrap_or(false)
}

/// Der Nullpunkt - legt ihn beim ersten Zugriff an.
///
/// pygame bekommt seinen Nullpunkt von `pygame.init()`. Wer hier ohne
/// [`init`] loslegt, soll trotzdem sinnvolle Zahlen sehen, darum startet
/// die Uhr beim ersten Blick auf sie.
fn nullpunkt() -> Option<Instant> {
    let mut s = START.lock().ok()?;
    if s.is_none() {
        *s = Some(Instant::now());
    }
    *s
}

// ---------------------------------------------------------------------------
//  get_ticks
// ---------------------------------------------------------------------------

/// Millisekunden seit [`init`] - Gegenstueck zu `pygame.time.get_ticks`.
///
/// Nach [`quit`] ist das `0`, bis wieder [`init`] gerufen wird. Ohne
/// vorheriges [`init`] laeuft die Uhr beim ersten Aufruf los und liefert
/// darum `0`.
///
/// **Der Wert ist auf ganze Millisekunden abgeschnitten**, nicht gerundet.
/// Wer damit Dauern misst, verliert also bis zu einer Millisekunde je
/// Messpunkt - bei zwei Messpunkten bis zu zwei. Das ist pygames
/// Verhalten und bleibt so; fuer feinere Messungen gibt es
/// [`get_ticks_f64`].
///
/// Der Hinweis steht hier, weil er beim Bau des Tonteils wirklich
/// zugeschlagen hat: dessen Zeitvergleiche brauchen an der unteren
/// Schranke fuenf Millisekunden Luft, sonst schlagen sie sporadisch fehl.
pub fn get_ticks() -> u64 {
    match nullpunkt() {
        None => 0,
        Some(t) => t.elapsed().as_millis() as u64,
    }
}

/// Wie [`get_ticks`], nur mit den Nachkommastellen.
///
/// **Das gibt es in pygame-ce 2.5.7 nicht** - dort ist `get_ticks` immer
/// ganzzahlig. Fuer alles, was feiner misst als eine Millisekunde, waere
/// das Runden aber ein Verlust, und hier kostet es nichts: `Instant` hat
/// die Aufloesung ohnehin. Wer pygame-treu bleiben will, nimmt
/// [`get_ticks`].
pub fn get_ticks_f64() -> f64 {
    match nullpunkt() {
        None => 0.0,
        Some(t) => t.elapsed().as_secs_f64() * 1000.0,
    }
}

// ---------------------------------------------------------------------------
//  Warten
// ---------------------------------------------------------------------------

/// Wartet und gibt den Rechner frei - Gegenstueck zu `pygame.time.wait`.
///
/// Gibt zurueck, wie viele Millisekunden wirklich vergangen sind; das kann
/// mehr sein als verlangt. Negative Werte gelten als `0` - gemessen:
/// `pygame.time.wait(-5)` gibt `0` zurueck und wartet nicht.
pub fn wait(millisekunden: i64) -> u64 {
    let ms = millisekunden.max(0) as u64;
    let start = Instant::now();
    if ms > 0 {
        std::thread::sleep(Duration::from_millis(ms));
    }
    start.elapsed().as_millis() as u64
}

/// Wartet genauer und verbrennt dafuer Rechenzeit - Gegenstueck zu
/// `pygame.time.delay`.
///
/// Zeilengetreu aus `accurate_delay` in `src_c/time.c`: alles bis auf den
/// Rest wird verschlafen, der Rest aktiv abgewartet.
///
/// ```text
/// delay = (ticks - 2) - (ticks % 12)
/// ```
///
/// Die `- 2` sind der Sicherheitsabstand, die `% 12` das Wegrunden auf
/// das, was der Schlaf ueberhaupt treffen kann. Der Rueckgabewert ist
/// wieder die wirklich vergangene Zeit.
pub fn delay(millisekunden: i64) -> u64 {
    if millisekunden <= 0 {
        return 0;
    }
    let ziel = millisekunden as u64;
    let start = Instant::now();
    if ziel >= SCHLECHTESTE_AUFLOESUNG {
        let schlaf = (ziel - 2) - (ziel % SCHLECHTESTE_AUFLOESUNG);
        if schlaf >= SCHLECHTESTE_AUFLOESUNG {
            std::thread::sleep(Duration::from_millis(schlaf));
        }
    }
    while (start.elapsed().as_millis() as u64) < ziel {
        std::hint::spin_loop();
    }
    start.elapsed().as_millis() as u64
}

// ---------------------------------------------------------------------------
//  set_timer
// ---------------------------------------------------------------------------

/// Ein Zeitgeber, der wiederholt dasselbe Ereignis in die Warteschlange
/// legen soll.
///
/// **Die Warteschlange liegt nicht hier**, sondern in
/// `rustgame-display` - Ton und Zeit sollen kein Fenster brauchen.
/// Deshalb fuehrt dieses Modul nur Buch, und `rustgame-display` holt
/// sich die faelligen Auftraege mit [`faellige_zeitgeber`] ab. Seine
/// `event::durchlauf` tut das vor jedem Blick in die Schlange, sodass
/// die Ereignisse auch der Sperrliste unterliegen.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Zeitgeber {
    /// Die Ereignisnummer, die hinterher in der Warteschlange landet
    /// (`pygame.USEREVENT` und aufwaerts).
    pub ereignis: u32,
    /// Abstand zwischen zwei Ereignissen in Millisekunden.
    pub millis: u32,
    /// Wie oft noch. `0` heisst: unbegrenzt.
    pub wiederholungen: u32,
    /// Wann das naechste Ereignis faellig ist - in Millisekunden seit
    /// [`init`], also in derselben Zeitrechnung wie [`get_ticks`].
    pub faellig: u64,
}

/// Alle laufenden Zeitgeber. Je Ereignisnummer hoechstens einer - so
/// macht es pygame auch: ein zweiter `set_timer` auf dieselbe Nummer
/// ersetzt den ersten.
static ZEITGEBER: Mutex<Vec<Zeitgeber>> = Mutex::new(Vec::new());

fn zeitgeber_alle_loeschen() {
    if let Ok(mut v) = ZEITGEBER.lock() {
        v.clear();
    }
}

/// Legt einen wiederkehrenden Zeitgeber an - Gegenstueck zu
/// `pygame.time.set_timer(event, millis, loops=0)`.
///
/// `millis <= 0` haelt einen laufenden Zeitgeber fuer dieses Ereignis an,
/// genau wie in pygame. `loops = 0` heisst unbegrenzt.
///
/// **Zugestellt wird anderswo.** Dieses Modul fuehrt nur Buch; die
/// Ereignisse reiht `rustgame-display` ein, indem es
/// [`faellige_zeitgeber`] abholt - einmal je Durchlauf, vor jedem Blick
/// in die Schlange. Das ist die ganze Schnittstelle, und sie haelt die
/// Richtung sauber: Zeit haengt nicht am Fenster.
pub fn set_timer(ereignis: u32, millis: i64, wiederholungen: u32) {
    let Ok(mut v) = ZEITGEBER.lock() else { return };
    v.retain(|z| z.ereignis != ereignis);
    if millis <= 0 {
        return;
    }
    let millis = millis.min(u32::MAX as i64) as u32;
    v.push(Zeitgeber {
        ereignis,
        millis,
        wiederholungen,
        faellig: get_ticks() + millis as u64,
    });
}

/// Welche Zeitgeber sind jetzt faellig?
///
/// **Diese Funktion bedient `rustgame-display`**, sobald es eine
/// Ereigniswarteschlange gibt: einmal je Durchlauf aufrufen, die
/// zurueckgegebenen Ereignisnummern in die Warteschlange legen, fertig.
/// Der naechste Faelligkeitszeitpunkt und die Restzahl der
/// Wiederholungen werden hier schon nachgezogen; abgelaufene Zeitgeber
/// verschwinden von selbst.
///
/// **Verschlafene Abstaende werden nachgeholt, nicht verschluckt.** Ein
/// Zeitgeber, der seit dem letzten Aufruf dreimal faellig war, meldet
/// sich dreimal. Das war hier einmal anders (einmal je Aufruf, mit der
/// Begruendung "so wie SDL es macht"), und das war schlicht falsch
/// gemessen: SDL laesst einen eigenen Strang laufen, der bei **jedem**
/// Abstand ein Ereignis einreiht, ganz gleich, wann das Programm das
/// naechste Mal hinsieht.
///
/// ```text
/// Zeitgeber alle 20 ms, sechsmal im Abstand von 30 ms abgefragt:
///   pygame    8 Ereignisse
///   vorher    6      <- eines je Abfrage
///   jetzt     8
/// ```
///
/// Ebenso wichtig: der naechste Termin wird **vom letzten Termin** aus
/// weitergezaehlt, nicht von jetzt. Sonst verschiebt sich der Takt um
/// die Abfragedauer, und ein 20-ms-Zeitgeber laeuft bei 30-ms-Abfragen
/// in Wahrheit auf 30 ms.
pub fn faellige_zeitgeber() -> Vec<u32> {
    let jetzt = get_ticks();
    let Ok(mut v) = ZEITGEBER.lock() else {
        return Vec::new();
    };
    let mut aus = Vec::new();
    v.retain_mut(|z| {
        // `millis` ist immer mindestens 1 - `set_timer` weist 0 und
        // Negatives ab -, die Schleife haelt also an.
        while jetzt >= z.faellig {
            aus.push(z.ereignis);
            z.faellig += z.millis as u64;
            if z.wiederholungen > 0 {
                z.wiederholungen -= 1;
                if z.wiederholungen == 0 {
                    return false;
                }
            }
        }
        true
    });
    aus
}

/// Alle laufenden Zeitgeber - fuer Tests und zum Nachsehen.
pub fn laufende_zeitgeber() -> Vec<Zeitgeber> {
    ZEITGEBER.lock().map(|v| v.clone()).unwrap_or_default()
}

// ---------------------------------------------------------------------------
//  Clock
// ---------------------------------------------------------------------------

/// Die Uhr aus `pygame.time.Clock` - haelt die Bildrate und misst sie.
///
/// Der uebliche Gebrauch:
///
/// ```no_run
/// # use rustgame_core::time::Clock;
/// let mut uhr = Clock::new();
/// loop {
///     // zeichnen ...
///     let dt = uhr.tick(60.0); // wartet, bis 1/60 Sekunde um ist
///     # let _ = dt; break;
/// }
/// ```
///
/// Aufbau und Rechnung sind aus `clock_tick_base` in `src_c/time.c`
/// uebernommen - inklusive der beiden Stellen, die Nachbauten gern
/// uebersehen: `get_fps` mittelt ueber **zehn** `tick`, und `get_rawtime`
/// misst die Zeit **vor** dem Warten.
#[derive(Debug, Clone)]
pub struct Clock {
    /// Wann zuletzt `tick` fertig war - in Millisekunden seit [`init`].
    last_tick: u64,
    /// Wie viele `tick` seit der letzten Bildratenrechnung.
    fps_count: u64,
    /// Wann zuletzt gerechnet wurde. `0` heisst: noch nie.
    fps_tick: u64,
    /// Die zuletzt gerechnete Bildrate.
    fps: f32,
    /// Zeit seit dem vorigen `tick`, **mit** Wartezeit.
    timepassed: u64,
    /// Zeit seit dem vorigen `tick`, **ohne** Wartezeit.
    rawpassed: u64,
}

impl Default for Clock {
    fn default() -> Clock {
        Clock::new()
    }
}

impl Clock {
    /// Eine frische Uhr. Alle Messwerte stehen auf null - gemessen:
    /// `pygame.time.Clock()` liefert `get_fps() == 0.0`,
    /// `get_time() == 0` und `get_rawtime() == 0`.
    pub fn new() -> Clock {
        Clock {
            last_tick: get_ticks(),
            fps_count: 0,
            fps_tick: 0,
            fps: 0.0,
            timepassed: 0,
            rawpassed: 0,
        }
    }

    /// Einen Durchlauf abschliessen und, wenn noetig, auf die Bildrate
    /// warten. Gibt die Millisekunden seit dem vorigen `tick` zurueck.
    ///
    /// `framerate = 0.0` heisst "nicht warten, nur messen".
    ///
    /// Gewartet wird mit [`wait`], also grob - genau wie `pygame`s `tick`,
    /// das dafuer `SDL_Delay` nimmt. Wer es genauer braucht, nimmt
    /// [`tick_busy_loop`](Self::tick_busy_loop).
    pub fn tick(&mut self, framerate: f32) -> u64 {
        self.tick_base(framerate, false)
    }

    /// Wie [`tick`](Self::tick), aber mit dem genauen Warten aus
    /// [`delay`]: der letzte Rest wird aktiv abgewartet statt verschlafen.
    ///
    /// Das haelt die Bildrate spuerbar ruhiger und kostet dafuer einen
    /// Kern. Gegenstueck zu `Clock.tick_busy_loop`.
    pub fn tick_busy_loop(&mut self, framerate: f32) -> u64 {
        self.tick_base(framerate, true)
    }

    /// Der gemeinsame Kern - `clock_tick_base` aus `src_c/time.c`.
    fn tick_base(&mut self, framerate: f32, genau: bool) -> u64 {
        if framerate != 0.0 {
            // pygame rechnet hier in `float` und schneidet dann ab; bei
            // 60 kommen so 16 heraus, nicht 17. Das zaehlt: ueber eine
            // Sekunde macht das mehr als ein Bild Unterschied.
            let endtime = ((1.0f32 / framerate) * 1000.0) as i64;
            self.rawpassed = get_ticks().saturating_sub(self.last_tick);
            let verzug = endtime - self.rawpassed as i64;
            if genau {
                delay(verzug);
            } else {
                wait(verzug);
            }
        }

        let nowtime = get_ticks();
        self.timepassed = nowtime.saturating_sub(self.last_tick);
        self.fps_count += 1;
        self.last_tick = nowtime;
        if framerate == 0.0 {
            // Ohne Bildrate wurde nicht gewartet - dann sind beide gleich.
            self.rawpassed = self.timepassed;
        }

        if self.fps_tick == 0 {
            // Der erste `tick` legt nur den Nullpunkt fest. pygame prueft
            // hier woertlich `if (!self->fps_tick)` - mit der Folge, dass
            // eine Uhr, die in der allerersten Millisekunde nach `init`
            // laeuft, diesen Zweig mehrfach nimmt und die Zaehlung immer
            // wieder von vorn beginnt. Das ist kein Versehen im Nachbau,
            // sondern steht genau so in `src_c/time.c`.
            self.fps_count = 0;
            self.fps_tick = nowtime;
        } else if self.fps_count >= 10 {
            // Genau zehn Aufrufe, dann einmal teilen und von vorn. Kein
            // gleitender Mittelwert - das ist der Unterschied, der
            // Nachbauten auffliegen laesst.
            self.fps = self.fps_count as f32 / ((nowtime - self.fps_tick) as f32 / 1000.0);
            self.fps_count = 0;
            self.fps_tick = nowtime;
        }
        self.timepassed
    }

    /// Die zuletzt gerechnete Bildrate - Mittel ueber die **letzten zehn**
    /// `tick`, nicht ueber alle.
    ///
    /// Vor dem elften `tick` steht hier `0.0`. Das ist kein Fehler,
    /// sondern pygames Verhalten: nachgemessen liefern die ersten zehn
    /// `tick` einer frischen Uhr `0.0`.
    pub fn get_fps(&self) -> f32 {
        self.fps
    }

    /// Millisekunden zwischen den beiden letzten `tick` - **mit** der
    /// Wartezeit.
    pub fn get_time(&self) -> u64 {
        self.timepassed
    }

    /// Millisekunden zwischen den beiden letzten `tick` - **ohne** die
    /// Wartezeit.
    ///
    /// Das ist die Zeit, die der Durchlauf wirklich gekostet hat. Wird
    /// `tick` ohne Bildrate gerufen, ist es dasselbe wie
    /// [`get_time`](Self::get_time).
    pub fn get_rawtime(&self) -> u64 {
        self.rawpassed
    }
}

impl std::fmt::Display for Clock {
    /// Genau wie `clock_str` in `src_c/time.c`: `<Clock(fps=60.00)>`.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "<Clock(fps={:.2})>", self.fps)
    }
}

// ---------------------------------------------------------------------------
//  Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    /// Die Uhr ist ein globaler Zustand, die Tests laufen nebeneinander -
    /// wer [`init`] oder [`quit`] anfasst, nimmt erst diese Sperre.
    static GLOBAL: Mutex<()> = Mutex::new(());

    // ----- get_ticks -------------------------------------------------------

    #[test]
    fn get_ticks_laeuft_und_steigt() {
        let _sperre = GLOBAL.lock().unwrap_or_else(|e| e.into_inner());
        init();
        let a = get_ticks();
        wait(20);
        let b = get_ticks();
        assert!(b >= a + 15, "von {a} auf {b} - da fehlt Zeit");
        assert!(b < a + 400, "von {a} auf {b} - das ist zu viel");
        quit();
    }

    /// Nach `quit` ist die Uhr aus und meldet `0` - so wie
    /// `pygame.time.get_ticks()` nach `pygame.quit()`. Gemessen.
    #[test]
    fn quit_setzt_die_uhr_zurueck() {
        let _sperre = GLOBAL.lock().unwrap_or_else(|e| e.into_inner());
        init();
        wait(15);
        assert!(get_ticks() > 0);
        quit();
        assert!(!get_init());
        assert_eq!(get_ticks_ohne_anlauf(), 0);
        // Und ein neues `init` faengt wieder klein an.
        init();
        assert!(get_ticks() < 50);
        quit();
    }

    /// `get_ticks` startet die Uhr beim ersten Blick - fuer diesen Test
    /// braucht es den Wert *ohne* dieses Anlaufen.
    fn get_ticks_ohne_anlauf() -> u64 {
        match START.lock().ok().and_then(|s| *s) {
            None => 0,
            Some(t) => t.elapsed().as_millis() as u64,
        }
    }

    #[test]
    fn get_ticks_f64_passt_zu_get_ticks() {
        let _sperre = GLOBAL.lock().unwrap_or_else(|e| e.into_inner());
        init();
        wait(10);
        let grob = get_ticks();
        let fein = get_ticks_f64();
        assert!((fein - grob as f64).abs() < 5.0, "{fein} gegen {grob}");
        quit();
    }

    // ----- wait / delay ----------------------------------------------------

    /// Negative und null: kein Warten, Rueckgabe `0`. Gemessen an pygame -
    /// `wait(-5)` und `delay(-5)` geben beide `0`.
    #[test]
    fn wait_und_delay_bei_null_und_negativ() {
        assert_eq!(wait(0), 0);
        assert_eq!(wait(-5), 0);
        assert_eq!(delay(0), 0);
        assert_eq!(delay(-5), 0);
    }

    #[test]
    fn wait_wartet_wirklich() {
        let start = Instant::now();
        let gemeldet = wait(30);
        let wirklich = start.elapsed().as_millis() as u64;
        assert!(gemeldet >= 28, "gemeldet {gemeldet}");
        assert!(wirklich >= 28, "wirklich {wirklich}");
        assert!(wirklich < 400, "wirklich {wirklich} - viel zu lang");
    }

    /// `delay` ist der genauere der beiden - es darf nie *zu frueh*
    /// zurueckkommen.
    #[test]
    fn delay_wartet_mindestens_so_lange_wie_verlangt() {
        for ziel in [1u64, 5, 13, 30] {
            let start = Instant::now();
            let gemeldet = delay(ziel as i64);
            let wirklich = start.elapsed().as_millis() as u64;
            assert!(
                wirklich >= ziel,
                "delay({ziel}) war nach {wirklich} ms zurueck"
            );
            assert!(gemeldet >= ziel, "delay({ziel}) meldete {gemeldet}");
            assert!(wirklich < ziel + 400, "delay({ziel}) brauchte {wirklich}");
        }
    }

    // ----- Clock -----------------------------------------------------------

    /// Eine frische Uhr steht auf null - gemessen an
    /// `pygame.time.Clock()`.
    #[test]
    fn frische_uhr_steht_auf_null() {
        let uhr = Clock::new();
        assert_eq!(uhr.get_fps(), 0.0);
        assert_eq!(uhr.get_time(), 0);
        assert_eq!(uhr.get_rawtime(), 0);
        assert_eq!(uhr.to_string(), "<Clock(fps=0.00)>");
    }

    /// Der Kern der Bildratenrechnung, ohne echte Zeit: `get_fps` bleibt
    /// **zehn** `tick` lang `0.0` und bekommt erst beim elften einen Wert.
    ///
    /// Genau so gemessen an echtem pygame: dort stehen die Aufrufe 0 bis 9
    /// auf `0.000`, Aufruf 10 liefert den ersten Wert.
    #[test]
    fn get_fps_mittelt_ueber_zehn_ticks() {
        let _sperre = GLOBAL.lock().unwrap_or_else(|e| e.into_inner());
        init();
        // Erst dann eine Uhr anlegen, wenn `get_ticks()` schon laeuft -
        // sonst ist `fps_tick` null und der Nullpunkt-Zweig greift immer
        // wieder. In pygame ist das genauso, dort liegt `pygame.init()`
        // nur laengst zurueck.
        wait(2);
        let mut uhr = Clock::new();
        for i in 0..10 {
            wait(1);
            uhr.tick(0.0);
            assert_eq!(
                uhr.get_fps(),
                0.0,
                "nach tick {i} darf noch nichts dastehen"
            );
        }
        wait(1);
        uhr.tick(0.0);
        assert!(
            uhr.get_fps() > 0.0 && uhr.get_fps().is_finite(),
            "der elfte tick muss eine Bildrate liefern, war {}",
            uhr.get_fps()
        );
        quit();
    }

    /// Und danach wird wieder von vorn gezaehlt, nicht gleitend gemittelt:
    /// die naechsten neun `tick` aendern den Wert nicht.
    #[test]
    fn get_fps_bleibt_zwischen_den_rechnungen_stehen() {
        let _sperre = GLOBAL.lock().unwrap_or_else(|e| e.into_inner());
        init();
        wait(2);
        let mut uhr = Clock::new();
        for _ in 0..11 {
            wait(1);
            uhr.tick(0.0);
        }
        let erster = uhr.get_fps();
        assert!(erster > 0.0 && erster.is_finite(), "war {erster}");
        for _ in 0..9 {
            uhr.tick(0.0);
            assert_eq!(uhr.get_fps(), erster, "dazwischen darf sich nichts aendern");
        }
        wait(1);
        uhr.tick(0.0);
        // Der zwanzigste tick rechnet neu - ob derselbe Wert herauskommt,
        // haengt an der Maschine; dass ueberhaupt gerechnet wird, nicht.
        assert!(uhr.get_fps() > 0.0);
        quit();
    }

    /// `tick` mit Bildrate wartet wirklich und gibt die vergangene Zeit
    /// zurueck.
    #[test]
    fn tick_haelt_die_bildrate() {
        let _sperre = GLOBAL.lock().unwrap_or_else(|e| e.into_inner());
        init();
        let mut uhr = Clock::new();
        uhr.tick(50.0); // der erste tick misst nur
        let start = Instant::now();
        let dt = uhr.tick(50.0); // 1/50 s = 20 ms
        let wirklich = start.elapsed().as_millis() as u64;
        assert!(wirklich >= 15, "nur {wirklich} ms gewartet");
        assert!(dt >= 15, "tick meldete {dt}");
        assert!(wirklich < 400, "{wirklich} ms - das ist kein Warten mehr");
        quit();
    }

    /// `get_rawtime` misst **ohne** die Wartezeit, `get_time` **mit**.
    /// Bei einer leeren Schleife ist `rawtime` darum nahe null, `time`
    /// nahe dem Bildabstand.
    #[test]
    fn rawtime_zaehlt_die_wartezeit_nicht_mit() {
        let _sperre = GLOBAL.lock().unwrap_or_else(|e| e.into_inner());
        init();
        let mut uhr = Clock::new();
        uhr.tick(50.0);
        uhr.tick(50.0);
        assert!(
            uhr.get_rawtime() <= 5,
            "rawtime {} - da ist die Wartezeit mit drin",
            uhr.get_rawtime()
        );
        assert!(
            uhr.get_time() >= 15,
            "time {} - da fehlt die Wartezeit",
            uhr.get_time()
        );
        quit();
    }

    /// Ohne Bildrate sind beide gleich - so steht es in `time.c`
    /// (`if (!framerate) self->rawpassed = self->timepassed;`).
    #[test]
    fn ohne_bildrate_sind_time_und_rawtime_gleich() {
        let _sperre = GLOBAL.lock().unwrap_or_else(|e| e.into_inner());
        init();
        let mut uhr = Clock::new();
        uhr.tick(0.0);
        wait(15);
        uhr.tick(0.0);
        assert_eq!(uhr.get_time(), uhr.get_rawtime());
        assert!(uhr.get_time() >= 12);
        quit();
    }

    /// pygame rechnet den Bildabstand in `float` und schneidet ab: bei 60
    /// sind das 16 ms, nicht 17. Der Test prueft die Rechnung selbst.
    #[test]
    fn bildabstand_wird_abgeschnitten_nicht_gerundet() {
        for (rate, soll) in [(60.0f32, 16i64), (50.0, 20), (30.0, 33), (3.0, 333)] {
            assert_eq!(
                ((1.0f32 / rate) * 1000.0) as i64,
                soll,
                "bei {rate} Bildern"
            );
        }
    }

    #[test]
    fn tick_busy_loop_wartet_auch() {
        let _sperre = GLOBAL.lock().unwrap_or_else(|e| e.into_inner());
        init();
        let mut uhr = Clock::new();
        uhr.tick_busy_loop(100.0);
        let start = Instant::now();
        uhr.tick_busy_loop(100.0); // 10 ms
        let wirklich = start.elapsed().as_millis() as u64;
        assert!(wirklich >= 9, "nur {wirklich} ms");
        assert!(wirklich < 400, "{wirklich} ms");
        quit();
    }

    // ----- set_timer -------------------------------------------------------

    #[test]
    fn set_timer_fuehrt_buch() {
        let _sperre = GLOBAL.lock().unwrap_or_else(|e| e.into_inner());
        init();
        set_timer(24, 5, 0);
        set_timer(25, 5, 2);
        assert_eq!(laufende_zeitgeber().len(), 2);

        // Ein zweiter Aufruf auf dieselbe Nummer ersetzt den ersten.
        set_timer(24, 50, 0);
        let laufend = laufende_zeitgeber();
        assert_eq!(laufend.len(), 2);
        assert_eq!(
            laufend.iter().find(|z| z.ereignis == 24).unwrap().millis,
            50
        );

        // `millis <= 0` haelt ihn an.
        set_timer(24, 0, 0);
        assert_eq!(laufende_zeitgeber().len(), 1);
        quit();
        assert!(laufende_zeitgeber().is_empty());
    }

    #[test]
    fn faellige_zeitgeber_zaehlen_die_wiederholungen_herunter() {
        let _sperre = GLOBAL.lock().unwrap_or_else(|e| e.into_inner());
        init();
        set_timer(30, 1, 2);
        assert!(faellige_zeitgeber().is_empty(), "noch nicht faellig");
        // Fuenf Millisekunden spaeter waren beide Wiederholungen faellig -
        // und beide kommen, nicht nur die erste.
        wait(5);
        assert_eq!(faellige_zeitgeber(), vec![30, 30]);
        // Zweimal war verlangt, also ist jetzt Schluss.
        wait(5);
        assert!(faellige_zeitgeber().is_empty());
        assert!(laufende_zeitgeber().is_empty());
        quit();
    }

    #[test]
    /// **Verschlafenes wird nachgeholt.** Dieser Test hiess einmal
    /// `verschlafener_zeitgeber_meldet_sich_nur_einmal` und pruefte das
    /// Gegenteil - die Annahme dahinter ("so macht es SDL") war an
    /// echtem pygame nie nachgemessen und falsch.
    fn verschlafener_zeitgeber_holt_nach() {
        let _sperre = GLOBAL.lock().unwrap_or_else(|e| e.into_inner());
        init();
        set_timer(31, 1, 0);
        wait(30); // dreissig Abstaende verpasst
        let faellig = faellige_zeitgeber();
        assert!(
            faellig.len() >= 25,
            "mindestens ein Ereignis je verpasstem Abstand, waren {}",
            faellig.len()
        );
        assert!(faellig.iter().all(|e| *e == 31));
        set_timer(31, 0, 0);
        quit();
    }
}
