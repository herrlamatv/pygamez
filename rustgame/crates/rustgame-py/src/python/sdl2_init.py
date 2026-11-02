"""`pygame._sdl2` - das Paket, das die Namen seiner Kinder mittraegt.

Gemessen hat `dir(pygame._sdl2)` **achtundvierzig** Eintraege: acht
Untermodule und vierzig Namen, die es mit `import *` aus `audio`, `sdl2`
und `video` hereinholt. `pygame._sdl2.Window` geht darum ohne das
Untermodul, und ein Programm darf sich darauf stuetzen.

## Warum das Python ist und nicht Rust

Der erste Versuch stand in `lib.rs`: beim Einrichten des Moduls ueber die
drei Kinder laufen und jeden Namen mit `getattr` holen. **Das haengt.**
`video.py` holt seine Klassen mit `__getattr__` nachtraeglich und schreibt
dafuer `import rustgame_py` - und waehrend `rustgame_py` noch eingerichtet
wird, steht es nicht in `sys.modules`. Der Import faengt also von vorne an
und wartet auf sich selbst.

Ein `__getattr__` auf **Modulebene** loest das: gefragt wird erst, wenn
jemand den Namen will, und dann ist alles da. `__dir__` zaehlt sie
trotzdem auf, damit die Abdeckungstabelle sie sieht - dieselbe
Unterscheidung wie bei `surfarray` und `_sprite`.
"""

# Die acht Untermodule. Sie werden in `lib.rs` als Attribute gesetzt; hier
# stehen sie nur, damit `__dir__` sie mitzaehlt.
_UNTER = (
    "audio",
    "controller",
    "controller_old",
    "mixer",
    "sdl2",
    "touch",
    "video",
    "window",
)

# Woher jeder weitergereichte Name kommt. Gemessen an `dir(pygame._sdl2)`:
# alles aus `audio`, `sdl2` und `video`, nichts aus den anderen fuenf.
_WOHER = {
    "audio": (
        "AUDIO_ALLOW_ANY_CHANGE",
        "AUDIO_ALLOW_CHANNELS_CHANGE",
        "AUDIO_ALLOW_FORMAT_CHANGE",
        "AUDIO_ALLOW_FREQUENCY_CHANGE",
        "AUDIO_F32",
        "AUDIO_F32LSB",
        "AUDIO_F32MSB",
        "AUDIO_S16",
        "AUDIO_S16LSB",
        "AUDIO_S16MSB",
        "AUDIO_S32",
        "AUDIO_S32LSB",
        "AUDIO_S32MSB",
        "AUDIO_S8",
        "AUDIO_U16",
        "AUDIO_U16LSB",
        "AUDIO_U16MSB",
        "AUDIO_U8",
        "AudioDevice",
        "get_audio_device_names",
    ),
    "sdl2": (
        "INIT_AUDIO",
        "INIT_EVENTS",
        "INIT_EVERYTHING",
        "INIT_GAMECONTROLLER",
        "INIT_HAPTIC",
        "INIT_JOYSTICK",
        "INIT_NOPARACHUTE",
        "INIT_TIMER",
        "INIT_VIDEO",
        "init_subsystem",
    ),
    # **`traceback` kommt aus `audio` und `mixer`, nicht aus `video`** -
    # gemessen: `dir(pygame._sdl2)` enthaelt den Namen, und von den acht
    # Kindern haben ihn genau diese zwei. Hier steht er bei `mixer`, weil
    # jeder Name nur einmal nachgeschlagen werden muss; in beiden Kindern
    # steht er selbst.
    "mixer": ("traceback",),
    "video": (
        # **`errorfnc` ist die Fehlerklasse** - `pygame._sdl2.sdl2.error`.
        # Hier stand daneben "also `pygame.error` selbst", und das war
        # nie gemessen: die Klasse erbt direkt von `RuntimeError` und ist
        # **keine** Unterklasse von `pygame.error`. SDLs Cython-Schicht
        # legt sie unter diesem zweiten Namen ab, damit ihre Pruefmakros
        # sie werfen koennen. Sie steht damit auch in `dir(pygame._sdl2)`,
        # und ein Name zuviel ist derselbe Fehler wie einer zuwenig.
        "errorfnc",
        "Image",
        "Renderer",
        "RendererDriverInfo",
        "SCALEQUALITY_BEST",
        "SCALEQUALITY_LINEAR",
        "SCALEQUALITY_NEAREST",
        "Texture",
        "WINDOWPOS_CENTERED",
        "WINDOWPOS_UNDEFINED",
        "Window",
        "get_drivers",
        "get_grabbed_window",
    ),
}

__all__ = sorted(set(_UNTER) | {n for ns in _WOHER.values() for n in ns} | {"error"})


def __dir__():
    return list(__all__)


def __getattr__(name):
    import importlib

    if name == "error":
        # **Nicht `rustgame_py.error`** - siehe `sdl2_sdl2.error`.
        return importlib.import_module("rustgame_py._sdl2.sdl2").error
    for modul, namen in _WOHER.items():
        if name in namen:
            return getattr(
                importlib.import_module("rustgame_py._sdl2." + modul), name
            )
    raise AttributeError(
        "module %r has no attribute %r" % (__name__, name)
    )
