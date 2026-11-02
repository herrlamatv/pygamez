"""`pygame._sdl2.video` - derselbe Zeichner unter seinem zweiten Namen.

pygame hat diese Klassen zweimal: als `pygame._render` und als
`pygame._sdl2.video`. Es sind **dieselben Klassen**, nicht zwei
Umsetzungen - `video_test.py` legt ein `video.Window` an und reicht es an
`video.Renderer`, waehrend `render_test.py` dasselbe mit `pygame.Window`
und `_render.Renderer` tut.

Deshalb steht hier kein Nachbau, sondern eine Durchreiche.
"""

__all__ = [
    "errorfnc",
    "Window",
    "Renderer",
    "Texture",
    "Image",
    "RendererDriverInfo",
    "error",
    "get_drivers",
    "get_grabbed_window",
    "SCALEQUALITY_NEAREST",
    "SCALEQUALITY_LINEAR",
    "SCALEQUALITY_BEST",
    "WINDOWPOS_CENTERED",
    "WINDOWPOS_UNDEFINED",
]

#: Wie beim Strecken geglaettet wird - SDLs drei Stufen.
SCALEQUALITY_NEAREST = 0
SCALEQUALITY_LINEAR = 1
SCALEQUALITY_BEST = 2


class RendererDriverInfo:
    """Was ein Zeichentreiber ueber sich sagt.

    rustgame hat genau einen, und der rechnet im Hauptspeicher.
    """

    # **Kein `__init__`.** Gemessen: `RendererDriverInfo()` geht bei
    # pygame ohne Argumente durch und hat danach **kein** einziges Feld -
    # `repr()` scheitert dort mit `AttributeError: 'RendererDriverInfo'
    # object has no attribute 'name'`. Die Felder setzt `get_drivers()`
    # von aussen, so wie Cython es dort tut. Hier stand ein `__init__`
    # mit fuenf Vorgabewerten; damit nahm die Klasse fuenf Argumente, die
    # pygames nicht nimmt. Gefunden hat das die Frage, was eine Klasse
    # sagt, wenn man sie mit siebzehn Argumenten ruft.

    def __repr__(self):
        return (
            "<RendererDriverInfo(name: %s, flags: %d, num_texture_formats: %d, "
            "max_texture_width: %d, max_texture_height: %d)>"
            % (
                self.name,
                self.flags,
                self.num_texture_formats,
                self.max_texture_width,
                self.max_texture_height,
            )
        )


def get_drivers():
    """Alle Zeichentreiber - hier ist es einer.

    Die Felder werden **von aussen** gesetzt, weil die Klasse selbst
    keinen `__init__` hat; siehe dort.
    """
    i = RendererDriverInfo()
    i.name = "software"
    i.flags = 0
    i.num_texture_formats = 1
    i.max_texture_width = 16384
    i.max_texture_height = 16384
    yield i


def __dir__():
    """Damit `dir()` auch die durchgereichten Namen zeigt.

    Ohne diese Zeile sieht man von aussen nur `__getattr__` selbst: ein
    Modul, das seine Namen erst beim Zugriff holt, hat sie nicht in
    seinem Woerterbuch. `api_coverage.py` zaehlt genau dieses Woerterbuch
    ab und meldete hier **null von neun**.

    **Und `globals()` gehoert dazu.** `sorted(__all__)` allein war wieder
    eine Liste von Hand: `traceback` und `errorfnc` stehen bei pygame in
    `dir(pygame._sdl2.video)`, hier standen sie nicht.
    """
    return sorted(set(__all__) | {n for n in globals() if not n.startswith("_")})


def __getattr__(name):
    """Holt die Namen erst beim Zugriff.

    Dieser Text laeuft waehrend der Einrichtung von `rustgame_py`; ein
    `from rustgame_py import ...` ganz oben liefe in einen Ring.
    """
    import rustgame_py

    if name in ("Renderer", "Texture", "Image"):
        return getattr(rustgame_py._render, name)
    if name == "Window":
        return rustgame_py.Window
    if name in ("error", "errorfnc"):
        # **Beide Namen, eine Klasse** - und zwar
        # `pygame._sdl2.sdl2.error`, **nicht** `pygame.error`; der
        # Unterschied ist gemessen und steht dort beschrieben.
        import importlib

        return importlib.import_module("rustgame_py._sdl2.sdl2").error
    if name == "_messagebox":
        # **Mit Unterstrich, und das ist gemessen.** `pygame._sdl2.video`
        # hat `messagebox` **nicht** - hier stand der Name ohne
        # Unterstrich, und die Probe `_sdl2.Untermodule` hat ihn als
        # einzigen Ueberschuss gemeldet. Ein Name zuviel ist derselbe
        # Fehler wie einer zuwenig: beide sagen etwas Falsches ueber die
        # Schnittstelle.
        return rustgame_py.display.message_box
    if name == "get_grabbed_window":
        # **Es steht in `window`, nicht in `display`.** Hier stand
        # `rustgame_py.display.get_grabbed_window`, und das gibt es dort
        # nicht - `api_coverage.py` meldete den Namen als "nur im Kern",
        # weil er aus Python einen `AttributeError` gab. Ein Zeiger auf
        # eine leere Stelle ist schlimmer als kein Zeiger: er sieht aus
        # wie eine Umsetzung.
        return rustgame_py.window.get_grabbed_window
    if name in ("WINDOWPOS_CENTERED", "WINDOWPOS_UNDEFINED"):
        return getattr(rustgame_py, name)
    raise AttributeError(
        "module %r has no attribute %r" % (__name__, name)
    )
