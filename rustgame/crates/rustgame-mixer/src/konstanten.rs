//! konstanten.rs
//! =============
//! Die festen Zahlen - alle aus echtem pygame-ce 2.5.7 gemessen oder aus
//! `src_c/mixer.h` abgelesen.

/// Die volle Lautstaerke, in SDL_mixers Stufen: `MIX_MAX_VOLUME`.
///
/// **Das ist der Grund fuer die krummen Werte, die `get_volume()`
/// liefert.** Lautstaerke ist innen kein Bruch, sondern eine ganze Zahl
/// von 0 bis 128. Gemessen: `set_volume(0.1)` gibt `0.09375` zurueck
/// (= 12/128), `set_volume(1.0/255.0)` gibt glatt `0.0`.
pub const MAX_VOLUME: i32 = 128;

/// Die Balance-Stufen von `Mix_SetPanning`: 0 bis 255 je Seite.
pub const MAX_PANNING: i32 = 255;

/// `PYGAME_MIXER_DEFAULT_FREQUENCY`.
pub const VORGABE_FREQUENZ: i32 = 44_100;

/// `PYGAME_MIXER_DEFAULT_SIZE` - negativ heisst vorzeichenbehaftet.
pub const VORGABE_GROESSE: i32 = -16;

/// `PYGAME_MIXER_DEFAULT_CHANNELS` - gemeint sind die Ausgabespuren
/// (Stereo), nicht die Mischkanaele.
pub const VORGABE_KANAELE: i32 = 2;

/// `PYGAME_MIXER_DEFAULT_CHUNKSIZE`.
pub const VORGABE_PUFFER: i32 = 512;

/// Wie viele Mischkanaele `init()` anlegt.
///
/// Gemessen: **8**, nicht 16. SDL_mixers `MIX_CHANNELS` ist zwar 8, aber
/// das ueberrascht regelmaessig, weil die Doku die Zahl nirgends nennt.
pub const VORGABE_STIMMEN: usize = 8;

/// Was **echtes pygame** bei `mixer.get_sdl_mixer_version()` meldet.
///
/// Gemessen in der venv des Projekts: SDL_mixer 2.8.1, und `linked=False`
/// liefert dasselbe.
///
/// **Diese Kiste meldet die Zahl nicht weiter.** Sie bindet kein
/// SDL_mixer ein, also gibt [`crate::mixer::get_sdl_mixer_version`]
/// bewusst `None` - die Begruendung steht dort. Der Wert bleibt hier
/// als das stehen, was er ist: eine Messung an echtem pygame, fuer den
/// Modulvergleich.
pub const SDL_MIXER_VERSION: (i32, i32, i32) = (2, 8, 1);

/// `pygame.NOEVENT` - was `get_endevent()` liefert, solange keines gesetzt
/// ist. Gemessen: `0`.
pub const NOEVENT: u32 = 0;

/// `pygame.USEREVENT`.
///
/// Steht hier nur, damit Beispiele und Tests ohne `rustgame-display`
/// auskommen; gemessen ist der Wert `32865`.
pub const USEREVENT: u32 = 32_865;
