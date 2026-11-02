"""`pygame._sdl2.sdl2` - SDLs Teilsysteme.

Neun Zahlen und ein Aufruf. Die Zahlen sind SDLs `SDL_INIT_*`-Flaggen, und
sie sind gemessen, nicht hergeleitet:

```text
INIT_TIMER              1
INIT_AUDIO             16
INIT_VIDEO             32
INIT_JOYSTICK         512
INIT_HAPTIC          4096
INIT_GAMECONTROLLER  8192
INIT_EVENTS         16384
INIT_NOPARACHUTE  1048576
INIT_EVERYTHING     62001
```

`INIT_EVERYTHING` ist dabei **nicht** die Summe aller anderen: 62001 ist
`0xF231`, und darin fehlt `INIT_NOPARACHUTE`. Das ist Absicht - der
"Fallschirm" ist SDLs alte Absturzbehandlung, und die will man nicht
mitgestartet haben. Wer die Zahl herleitet statt sie abzulesen, liegt
daneben.
"""

__all__ = [
    "INIT_AUDIO",
    "INIT_EVENTS",
    "INIT_EVERYTHING",
    "INIT_GAMECONTROLLER",
    "INIT_HAPTIC",
    "INIT_JOYSTICK",
    "INIT_NOPARACHUTE",
    "INIT_TIMER",
    "INIT_VIDEO",
    "error",
    "init_subsystem",
]

INIT_TIMER = 1
INIT_AUDIO = 16
INIT_VIDEO = 32
INIT_JOYSTICK = 512
INIT_HAPTIC = 4096
INIT_GAMECONTROLLER = 8192
INIT_EVENTS = 16384
INIT_NOPARACHUTE = 1048576
INIT_EVERYTHING = 62001



class error(RuntimeError):
    """SDLs Fehler - **eine eigene Klasse, nicht `pygame.error`.**

    Hier stand ein `__getattr__`, das `rustgame_py.error` zurueckgab, und
    daneben der Vermerk, `pygame._sdl2.sdl2.error` sei "`pygame.error`
    selbst". Gemessen ist das falsch:

    ```text
    pygame._sdl2.sdl2.error.__mro__   (error, RuntimeError, Exception, ...)
    issubclass(.., pygame.error)      False
    pygame._sdl2.Texture(None, (1,1)) pygame._sdl2.sdl2.error
      isinstance(.., pygame.error)    False
    error()                           error('')
    error("x").args                   ('x',)
    error(*17 Argumente)              TypeError: __init__() takes at
                                      most 2 positional arguments
    ```

    Es ist also eine zweite, unabhaengige Fehlerklasse, die direkt von
    `RuntimeError` erbt - wer `except pygame.error` schreibt, faengt sie
    **nicht**. Gefunden hat das die Frage, was jede Klasse sagt, wenn man
    sie mit siebzehn Argumenten ruft: pygame antwortete mit einem
    Argumentfehler, hier kam ein fertiger Fehler heraus, weil
    `pygame.error` wie jede `Exception` beliebig viele Argumente nimmt.

    `errorfnc` im Elternmodul ist derselbe Name fuer dieselbe Klasse.
    """

    def __init__(self, msg=None):
        # Ohne Argument ist die Meldung leer - gemessen `error('')`,
        # nicht `error()`.
        super().__init__("" if msg is None else msg)


def __dir__():
    """`__all__` **und** was das Modul selbst eingefuehrt hat.

    Hier stand `sorted(__all__)` - eine von Hand gepflegte Liste, und
    damit genau die Sorte Messgeraet, an der dieses Projekt schon zehn
    blinde Flecken hatte. Ein `import` am Kopf stand dann im Woerterbuch
    des Moduls, aber nicht in `dir()`.

    `globals()` ist die Antwort, die sich selbst nachfuehrt.
    """
    return sorted(set(__all__) | {n for n in globals() if not n.startswith("_")})


def init_subsystem(flags):
    """Faehrt einzelne Teilsysteme hoch - gibt `None`.

    **Es gibt hier kein SDL**, also auch keine Teilsysteme, die getrennt
    hochzufahren waeren. Was die Flaggen bei pygame bewirken, tut hier
    `pygame.init()` beziehungsweise das jeweilige `init()` des Moduls
    (`display.init`, `mixer.init`, `joystick.init`, ...).

    Gemessen gibt pygames `init_subsystem(INIT_VIDEO)` ebenfalls `None`
    und keinen Hinweis darauf, ob es geklappt hat - der Rueckgabewert ist
    also kein Unterschied, den ein Programm sehen koennte.
    """
    # pygame liest hier eine ganze Zahl; alles andere gibt
    # `TypeError: an integer is required`. Siehe `pypm._ganze_zahl`.
    # **Ein Wahrheitswert ist eine ganze Zahl** - `False` ist hier
    # die Vorgabe, und der erste Entwurf lehnte sie ab.
    if not isinstance(flags, int):
        raise TypeError("an integer is required")
    _ = int(flags)
    return None
