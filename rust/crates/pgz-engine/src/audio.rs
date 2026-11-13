//! audio.rs
//! ========
//! Gegenstueck zu `audio.py`: 17 synthetisierte Effekte, frei waehlbare
//! Toene und Musikschleifen aus mehreren Stimmen.
//!
//! Die Klaenge werden wie in Python **zur Laufzeit gerechnet** (kurze
//! Wellenformen mit Huellkurve) und als 16-Bit-PCM an rustgames Mixer
//! gegeben - keine Klangdateien neben der .exe. Die Rechenvorschrift ist
//! Zeile fuer Zeile die von `audio.py`, das Rauschen kommt aus demselben
//! Zufall ([`crate::py`]).
//!
//! Alles ist abgesichert: ohne Tongeraet laeuft das Spiel stumm weiter.

use std::cell::RefCell;
use std::collections::HashMap;

use pgz_core::settings::Settings;
use rustgame_mixer::{mixer, Channel, Sound};

use crate::py::{self, Random};

/// Eine Effekt-Beschreibung (`audio._SPECS`).
#[derive(Debug, Clone, Copy)]
struct Spec {
    f0: f64,
    f1: Option<f64>,
    dur: f64,
    wave: &'static str,
    vol: f64,
}

const SPECS: &[(&str, Spec)] = &[
    ("click", Spec { f0: 880.0, f1: None, dur: 0.05, wave: "square", vol: 0.25 }),
    ("select", Spec { f0: 700.0, f1: None, dur: 0.05, wave: "sine", vol: 0.25 }),
    ("eat", Spec { f0: 660.0, f1: Some(1320.0), dur: 0.09, wave: "square", vol: 0.30 }),
    ("bounce", Spec { f0: 440.0, f1: None, dur: 0.05, wave: "square", vol: 0.30 }),
    ("point", Spec { f0: 520.0, f1: Some(1040.0), dur: 0.18, wave: "square", vol: 0.35 }),
    ("shoot", Spec { f0: 900.0, f1: Some(300.0), dur: 0.12, wave: "saw", vol: 0.28 }),
    ("explode", Spec { f0: 200.0, f1: None, dur: 0.22, wave: "noise", vol: 0.40 }),
    ("hit", Spec { f0: 160.0, f1: None, dur: 0.18, wave: "square", vol: 0.40 }),
    ("rotate", Spec { f0: 600.0, f1: None, dur: 0.04, wave: "sine", vol: 0.22 }),
    ("lock", Spec { f0: 200.0, f1: None, dur: 0.07, wave: "square", vol: 0.30 }),
    ("line", Spec { f0: 400.0, f1: Some(1200.0), dur: 0.25, wave: "square", vol: 0.40 }),
    ("merge", Spec { f0: 500.0, f1: Some(760.0), dur: 0.10, wave: "sine", vol: 0.30 }),
    ("move", Spec { f0: 330.0, f1: None, dur: 0.03, wave: "square", vol: 0.18 }),
    ("gameover", Spec { f0: 440.0, f1: Some(120.0), dur: 0.50, wave: "saw", vol: 0.40 }),
    ("win", Spec { f0: 520.0, f1: Some(1300.0), dur: 0.45, wave: "square", vol: 0.40 }),
    ("powerup", Spec { f0: 600.0, f1: Some(1200.0), dur: 0.18, wave: "square", vol: 0.32 }),
    ("level", Spec { f0: 440.0, f1: Some(1100.0), dur: 0.32, wave: "square", vol: 0.38 }),
];

/// Reservierte Musik-Kanaele 0..3 (`audio.MUSIC_CHANNELS`).
pub const MUSIC_CHANNELS: usize = 4;

/// Eine Musik-Stimme: Mono-Abtastwerte und relative Lautstaerke.
pub type Voice = (Vec<i16>, f64);

#[derive(Default)]
struct Music {
    owner: Option<usize>,
    channels: Vec<Channel>,
    sounds: Vec<Sound>,
    vols: Vec<f64>,
    paused: bool,
    volume: f64,
    sound_on: bool,
    base_volume: f64,
}

#[derive(Default)]
struct State {
    available: bool,
    rate: i32,
    channels: i32,
    cache: HashMap<&'static str, Option<Sound>>,
    tone_cache: HashMap<(i64, i64, String), Option<Sound>>,
    note_cache: HashMap<(i64, usize, String, i64, i64, i32), Vec<i16>>,
    music: Music,
}

thread_local! {
    static ST: RefCell<State> = RefCell::new(volume_default());
}

fn volume_default() -> State {
    State { music: Music { volume: 1.0, base_volume: 0.6, sound_on: true, ..Default::default() }, ..Default::default() }
}

fn with<R>(f: impl FnOnce(&mut State) -> R) -> R {
    ST.with(|s| f(&mut s.borrow_mut()))
}

/// `audio.init()` - Mixer starten und alle Effekte vorab erzeugen.
pub fn init() {
    let ok = (|| -> Option<(i32, i32)> {
        if mixer::get_init().is_none() {
            mixer::init(mixer::Einstellungen::neu(44100, -16, 1, 512)).ok()?;
        }
        mixer::set_num_channels(16).ok()?;
        let (f, _fmt, ch) = mixer::get_init()?;
        Some((f, ch))
    })();
    let Some((rate, channels)) = ok else {
        with(|s| s.available = false);
        return;
    };
    with(|s| {
        s.available = true;
        s.rate = rate;
        s.channels = channels;
    });
    for (name, spec) in SPECS {
        let snd = build(spec);
        with(|s| s.cache.insert(name, snd));
    }
}

/// `pygame.mixer.quit()` beim Beenden: Musik aus, Ausgabe schliessen.
///
/// Muss laufen, solange das Programm noch lebt. Sonst baut erst der
/// Prozess-Abbau den Thread-lokalen Mixer-Zustand ab - dann ist der
/// Audio-Thread schon weg, und das Abraeumen endet in einer Panik.
pub fn shutdown() {
    #[cfg(windows)]
    xinput::aus();
    music_stop(None);
    with(|s| {
        s.music.channels.clear();
        s.available = false;
    });
    if mixer::get_init().is_some() {
        mixer::quit();
    }
}

fn to_sound(samples: &[i16], channels: i32) -> Option<Sound> {
    let mut bytes = Vec::with_capacity(samples.len() * 2 * channels.max(1) as usize);
    for v in samples {
        for _ in 0..channels.max(1) {
            bytes.extend_from_slice(&v.to_le_bytes());
        }
    }
    Sound::from_buffer(&bytes).ok()
}

fn build(spec: &Spec) -> Option<Sound> {
    let (rate, channels) = with(|s| (s.rate, s.channels));
    let freq_hz = rate as f64;
    let n = 1.max((freq_hz * spec.dur) as i64);
    let f0 = spec.f0;
    let f1 = spec.f1.unwrap_or(f0);
    let amp = spec.vol * 32767.0;
    let att = 1.max((n as f64 * 0.05) as i64);
    let rel = 1.max((n as f64 * 0.20) as i64);
    let mut buf: Vec<i16> = Vec::with_capacity(n as usize);
    let mut phase = 0.0f64;
    for i in 0..n {
        let frac = i as f64 / n as f64;
        let f = f0 + (f1 - f0) * frac;
        phase += f / freq_hz;
        let p = py::fpmod(phase, 1.0);
        let val = match spec.wave {
            "sine" => (2.0 * py::PI * p).sin(),
            "square" => {
                if p < 0.5 {
                    1.0
                } else {
                    -1.0
                }
            }
            "saw" => 2.0 * p - 1.0,
            _ => py::uniform(-1.0, 1.0),
        };
        let env = 1.0f64.min(i as f64 / att as f64).min((n - i) as f64 / rel as f64);
        let s = (val.clamp(-1.0, 1.0) * amp * env) as i64;
        buf.push(s.clamp(i16::MIN as i64, i16::MAX as i64) as i16);
    }
    to_sound(&buf, channels)
}

/// `audio.play(name, settings)` - Effekt, falls Ton an.
pub fn play(name: &str, settings: Option<&Settings>) {
    if !with(|s| s.available) {
        return;
    }
    if let Some(st) = settings {
        if !st.sound {
            return;
        }
    }
    let snd = with(|s| s.cache.get(name).cloned().flatten());
    let Some(snd) = snd else { return };
    let vol = settings.map(|s| s.volume).unwrap_or(0.6);
    snd.set_volume(vol.clamp(0.0, 1.0));
    let _ = snd.play(0, 0, 0);
}

/// `audio.tone(freq, dur, settings, wave, vol)` - frei waehlbarer Ton.
pub fn tone(freq: f64, dur: f64, settings: Option<&Settings>, wave: &'static str, vol: f64) {
    if !with(|s| s.available) {
        return;
    }
    if let Some(st) = settings {
        if !st.sound {
            return;
        }
    }
    let key = (
        (py::round_to(freq, 1) * 10.0).round() as i64,
        (py::round_to(dur, 3) * 1000.0).round() as i64,
        wave.to_string(),
    );
    let vorhanden = with(|s| s.tone_cache.get(&key).cloned());
    let snd = match vorhanden {
        Some(s) => s,
        None => {
            let s = build(&Spec { f0: freq, f1: None, dur, wave, vol: 1.0 });
            with(|st| st.tone_cache.insert(key, s.clone()));
            s
        }
    };
    let Some(snd) = snd else { return };
    let v = settings.map(|s| s.volume).unwrap_or(0.6);
    snd.set_volume((v * vol).clamp(0.0, 1.0));
    let _ = snd.play(0, 0, 0);
}

/// `audio.rumble(settings, ms, strong=0.6, weak=0.4)` - Gamepad-Vibration,
/// falls Haptik an ist (ohne Einstellungen: immer, wie in Python).
///
/// Python nimmt pygame/SDL (`Joystick.rumble`), das unter Windows fuer
/// Xbox-kompatible Controller XInput benutzt - hier direkt XInput: linker
/// Motor `strong`, rechter `weak`, nach `ms` wieder aus. Ohne Controller
/// (oder im Pruefstand) passiert nichts.
pub fn rumble(settings: Option<&Settings>, ms: i32) {
    if settings.is_some_and(|s| !s.haptik) || py::deterministic() {
        return;
    }
    #[cfg(windows)]
    xinput::vibrieren(0.6, 0.4, ms);
    #[cfg(not(windows))]
    let _ = ms;
}

/// Diagnose: Zahl der angeschlossenen XInput-Controller (`None` ohne XInput).
pub fn gamepads() -> Option<usize> {
    #[cfg(windows)]
    {
        xinput::verbunden()
    }
    #[cfg(not(windows))]
    {
        None
    }
}

#[cfg(windows)]
mod xinput {
    use std::ffi::c_void;
    use std::sync::atomic::{AtomicU64, Ordering};
    use std::sync::OnceLock;

    #[repr(C)]
    struct Vibration {
        links: u16,
        rechts: u16,
    }

    type SetState = unsafe extern "system" fn(u32, *mut Vibration) -> u32;
    type GetState = unsafe extern "system" fn(u32, *mut [u8; 16]) -> u32;

    #[link(name = "kernel32")]
    extern "system" {
        fn LoadLibraryW(name: *const u16) -> isize;
        fn GetProcAddress(module: isize, name: *const u8) -> *const c_void;
    }

    /// XInput-Funktionen (einmal geladen; None ohne XInput).
    fn funktionen() -> Option<(SetState, GetState)> {
        static F: OnceLock<Option<(SetState, GetState)>> = OnceLock::new();
        *F.get_or_init(|| {
            for dll in ["xinput1_4.dll", "xinput9_1_0.dll"] {
                let name: Vec<u16> = dll.encode_utf16().chain(std::iter::once(0)).collect();
                // SAFETY: LoadLibraryW/GetProcAddress mit nullterminierten
                // Namen; die Adressen haben die dokumentierten Signaturen.
                unsafe {
                    let m = LoadLibraryW(name.as_ptr());
                    if m == 0 {
                        continue;
                    }
                    let set = GetProcAddress(m, c"XInputSetState".as_ptr().cast());
                    let get = GetProcAddress(m, c"XInputGetState".as_ptr().cast());
                    if !set.is_null() && !get.is_null() {
                        return Some((std::mem::transmute::<*const c_void, SetState>(set), std::mem::transmute::<*const c_void, GetState>(get)));
                    }
                }
            }
            None
        })
    }

    /// Zaehler: nur die juengste Vibration darf die Motoren wieder abschalten.
    static RUNDE: AtomicU64 = AtomicU64::new(0);
    /// Wurde je vibriert? (Sonst muss beim Beenden nichts geladen werden.)
    static BENUTZT: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

    /// Beim Beenden: alle Motoren aus (ein Nachlauf-Thread kaeme zu spaet).
    pub fn aus() {
        if !BENUTZT.load(Ordering::SeqCst) {
            return;
        }
        let Some((set, _)) = funktionen() else {
            return;
        };
        RUNDE.fetch_add(1, Ordering::SeqCst);
        for i in 0..4u32 {
            let mut v = Vibration { links: 0, rechts: 0 };
            // SAFETY: v lebt fuer die Dauer des Aufrufs.
            unsafe { set(i, &mut v) };
        }
    }

    /// Angeschlossene Controller (Plaetze 0..4).
    fn pads(get: GetState) -> Vec<u32> {
        (0..4u32)
            .filter(|&i| {
                let mut zustand = [0u8; 16];
                // SAFETY: XInputGetState schreibt hoechstens XINPUT_STATE (16 Byte).
                unsafe { get(i, &mut zustand) == 0 }
            })
            .collect()
    }

    pub fn verbunden() -> Option<usize> {
        funktionen().map(|(_, get)| pads(get).len())
    }

    pub fn vibrieren(strong: f64, weak: f64, ms: i32) {
        let Some((set, get)) = funktionen() else {
            return;
        };
        let pads = pads(get);
        if pads.is_empty() {
            return;
        }
        BENUTZT.store(true, Ordering::SeqCst);
        let motor =|v: f64| (v.clamp(0.0, 1.0) * 65535.0) as u16;
        for &i in &pads {
            let mut v = Vibration { links: motor(strong), rechts: motor(weak) };
            // SAFETY: v lebt fuer die Dauer des Aufrufs.
            unsafe { set(i, &mut v) };
        }
        let runde = RUNDE.fetch_add(1, Ordering::SeqCst) + 1;
        let dauer = std::time::Duration::from_millis(ms.max(0) as u64);
        std::thread::spawn(move || {
            std::thread::sleep(dauer);
            if RUNDE.load(Ordering::SeqCst) == runde {
                for i in pads {
                    let mut aus = Vibration { links: 0, rechts: 0 };
                    // SAFETY: wie oben.
                    unsafe { set(i, &mut aus) };
                }
            }
        });
    }
}

// ---------------------------------------------------------------------------
//  Musik-Schleifen
// ---------------------------------------------------------------------------

fn reserve_music_channels() -> bool {
    let (have, avail) = with(|s| (!s.music.channels.is_empty(), s.available));
    if have || !avail {
        return have;
    }
    let chans = (|| -> Option<Vec<Channel>> {
        if mixer::get_num_channels().ok()? < 16 + MUSIC_CHANNELS {
            mixer::set_num_channels(16 + MUSIC_CHANNELS).ok()?;
        }
        mixer::set_reserved(MUSIC_CHANNELS).ok()?;
        (0..MUSIC_CHANNELS).map(|i| Channel::new(i).ok()).collect()
    })();
    let ok = chans.as_ref().map(|c| !c.is_empty()).unwrap_or(false);
    with(|s| s.music.channels = chans.unwrap_or_default());
    ok
}

/// `audio.mixer_rate()` - Abtastrate des Mixers.
pub fn mixer_rate() -> i32 {
    let r = with(|s| s.rate);
    if r > 0 {
        r
    } else {
        44100
    }
}

/// `audio.note_bytes(freq, dur, wave, vol, decay)` - eine Note als Mono-Werte.
pub fn note(freq: f64, dur: f64, wave: &str, vol: f64, decay: f64) -> Vec<i16> {
    let rate = mixer_rate();
    let n = 1.max((rate as f64 * dur) as i64) as usize;
    let key = (
        (py::round_to(freq, 2) * 100.0).round() as i64,
        n,
        wave.to_string(),
        (py::round_to(vol, 3) * 1000.0).round() as i64,
        (py::round_to(decay, 2) * 100.0).round() as i64,
        rate,
    );
    if let Some(d) = with(|s| s.note_cache.get(&key).cloned()) {
        return d;
    }
    let rate_f = rate as f64;
    let amp = vol.clamp(0.0, 1.0) * 32767.0;
    let mut rng = Random::new(((freq * 100.0) as i64 + n as i64).unsigned_abs());
    let att = 1.max((rate_f * 0.004) as i64);
    let rel = 1.max(((n / 3) as i64).min((rate_f * 0.03) as i64));
    let mut phase = 0.0f64;
    let step = freq / rate_f;
    let mut buf = Vec::with_capacity(n);
    for i in 0..n {
        let tt = i as f64 / rate_f;
        let (val, mut env) = match wave {
            "kick" => {
                phase += (150.0 * (-tt * 28.0).exp() + 45.0) / rate_f;
                ((2.0 * py::PI * phase).sin(), (-tt * 9.0).exp())
            }
            "snare" => {
                phase += 190.0 / rate_f;
                (
                    0.65 * rng.uniform(-1.0, 1.0) + 0.35 * (2.0 * py::PI * phase).sin(),
                    (-tt * 18.0).exp(),
                )
            }
            "hat" => (rng.uniform(-1.0, 1.0), (-tt * 45.0).exp()),
            _ => {
                phase += step;
                let p = py::fpmod(phase, 1.0);
                let v = match wave {
                    "sine" => (2.0 * py::PI * p).sin(),
                    "saw" => 2.0 * p - 1.0,
                    "triangle" => {
                        if p < 0.5 {
                            4.0 * p - 1.0
                        } else {
                            3.0 - 4.0 * p
                        }
                    }
                    "noise" => rng.uniform(-1.0, 1.0),
                    _ => {
                        if p < 0.5 {
                            1.0
                        } else {
                            -1.0
                        }
                    }
                };
                (v, if decay > 0.0 { (-tt * decay).exp() } else { 1.0 })
            }
        };
        env *= 1.0f64.min(i as f64 / att as f64).min((n as i64 - i as i64) as f64 / rel as f64);
        let s = (val.clamp(-1.0, 1.0) * amp * env) as i64;
        buf.push(s.clamp(i16::MIN as i64, i16::MAX as i64) as i16);
    }
    with(|s| s.note_cache.insert(key, buf.clone()));
    buf
}

/// `audio.render_voice(notes, length)` - Noten an ihre Stelle setzen.
pub fn render_voice(notes: &[(f64, Vec<i16>)], length: f64) -> Vec<i16> {
    let rate = mixer_rate() as f64;
    let total = (rate * length) as usize;
    let mut out = vec![0i16; total];
    for (start, data) in notes {
        let pos = (rate * start) as i64;
        if pos < 0 || pos as usize >= total {
            continue;
        }
        let pos = pos as usize;
        let n = data.len().min(total - pos);
        out[pos..pos + n].copy_from_slice(&data[..n]);
    }
    out
}

fn apply_music_volume() {
    with(|s| {
        let base = if s.music.sound_on { s.music.base_volume } else { 0.0 };
        for (ch, vol) in s.music.channels.iter().zip(s.music.vols.iter()) {
            ch.set_volume((base * vol * s.music.volume).clamp(0.0, 1.0));
        }
    });
}

/// `audio.music_play(voices, owner, settings, volume)`.
///
/// `owner` ist eine Kennung des Spiels (die Adresse des Spielobjekts).
pub fn music_play(voices: &[Voice], owner: usize, settings: Option<&Settings>, volume: f64) -> bool {
    music_stop(None);
    if !with(|s| s.available) || !reserve_music_channels() {
        return false;
    }
    let channels = with(|s| s.channels);
    let mut sounds = Vec::new();
    let mut vols = Vec::new();
    for (mono, vol) in voices.iter().take(MUSIC_CHANNELS) {
        if let Some(snd) = to_sound(mono, channels) {
            sounds.push(snd);
            vols.push(*vol);
        }
    }
    with(|s| {
        s.music.owner = Some(owner);
        s.music.sounds = sounds.clone();
        s.music.vols = vols;
        s.music.paused = false;
        s.music.volume = volume;
        s.music.sound_on = settings.map(|s| s.sound).unwrap_or(true);
        s.music.base_volume = settings.map(|s| s.volume).unwrap_or(0.6);
    });
    apply_music_volume();
    let chans = with(|s| s.music.channels.clone());
    for (ch, snd) in chans.iter().zip(sounds.iter()) {
        let _ = ch.play(snd, -1, 0, 0);
    }
    !sounds.is_empty()
}

/// `audio.music_stop(owner=None)`
pub fn music_stop(owner: Option<usize>) {
    let cur = with(|s| s.music.owner);
    if owner.is_some() && cur != owner {
        return;
    }
    let chans = with(|s| s.music.channels.clone());
    for ch in &chans {
        ch.stop();
    }
    with(|s| {
        s.music.owner = None;
        s.music.sounds.clear();
        s.music.vols.clear();
        s.music.paused = false;
    });
}

/// `audio.music_set_volume(volume)`
pub fn music_set_volume(volume: f64) {
    with(|s| s.music.volume = volume.clamp(0.0, 1.0));
    apply_music_volume();
}

/// `audio.music_playing(owner=None)`
pub fn music_playing(owner: Option<usize>) -> bool {
    with(|s| !s.music.sounds.is_empty() && (owner.is_none() || s.music.owner == owner))
}

/// `audio.music_tick(current)` - die Musik folgt ihrem Besitzer.
pub fn music_tick(current: Option<usize>, paused: bool, settings: &Settings) {
    let owner = with(|s| s.music.owner);
    let Some(owner) = owner else { return };
    if current != Some(owner) {
        music_stop(None);
        return;
    }
    let chans = with(|s| {
        s.music.sound_on = settings.sound;
        s.music.base_volume = settings.volume;
        if paused != s.music.paused {
            s.music.paused = paused;
            Some(s.music.channels.clone())
        } else {
            None
        }
    });
    if let Some(chans) = chans {
        for ch in &chans {
            if paused {
                ch.pause();
            } else {
                ch.unpause();
            }
        }
    }
    apply_music_volume();
}
