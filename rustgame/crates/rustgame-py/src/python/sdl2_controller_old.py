"""`pygame._sdl2.controller_old` - die alte Fassung des Spielgeraet-Moduls.

pygame fuehrt beide: `pygame._sdl2.controller` ist die heutige, dieses hier
die von vorher. Sie unterscheiden sich in vier Namen, und `Controller` ist
in beiden eine **eigene** Klasse - gemessen:

```text
_sdl2.controller_old.Controller is _sdl2.controller.Controller   False
```

Was nur hier steht:

```text
get_eventstate()     True      ob Ereignisse gemeldet werden
set_eventstate(an)   None
name_forindex(0)     None      der Name zu einer Nummer
update()             None      "wird vom Ereignislauf selbst gerufen"
```

Und was in beiden steht, verhaelt sich gleich: `get_count()` ist 0,
`get_init()` nach `init()` wahr, `is_controller(0)` falsch - auf einer
Maschine ohne Spielgeraet.

## Warum eine eigene Klasse und kein Zweitname

Weil pygame es so hat. Ein `Controller = controller.Controller` waere
bequem und bei `isinstance` falsch: wer ein Geraet mit der alten Klasse
anlegt und mit der neuen prueft, bekommt bei pygame `False`. Zwei Klassen,
zwei Antworten.
"""

__all__ = [
    "Controller",
    "error",
    "get_count",
    "get_eventstate",
    "get_init",
    "init",
    "is_controller",
    "name_forindex",
    "quit",
    "set_eventstate",
    "update",
]

# **Kein eigener Schalter mehr.** Er stand hier, und er war einer zu viel:
# beide Module teilen ein Teilsystem - siehe `_neu()`.
_ereignisse = [True]


def _pg():
    import rustgame_py

    return rustgame_py


def __getattr__(name):
    if name == "error":
        return _pg().error
    raise AttributeError(
        "module %r has no attribute %r" % (__name__, name)
    )


def __dir__():
    """`__all__` **und** was das Modul selbst eingefuehrt hat.

    Hier stand `sorted(__all__)` - eine von Hand gepflegte Liste, und
    damit genau die Sorte Messgeraet, an der dieses Projekt schon zehn
    blinde Flecken hatte. Ein `import` am Kopf stand dann im Woerterbuch
    des Moduls, aber nicht in `dir()`.

    `globals()` ist die Antwort, die sich selbst nachfuehrt.
    """
    return sorted(set(__all__) | {n for n in globals() if not n.startswith("_")})


def _neu():
    """Die **neue** Fassung des Moduls - sie fuehrt den Schalter.

    **Beide Module teilen ein Teilsystem**, und das ist gemessen:

    ```text
    controller.init()        ->  controller_old.get_init()  True
    controller.quit()        ->  controller_old.get_init()  False
    controller_old.init()    ->  controller.get_init()      True
    ```

    Hier standen zwei **unabhaengige** Schalter, und das fiel keiner Probe
    auf: beide Module sahen fuer sich richtig aus. Gefunden hat es erst
    eine Gegenprobe, in der die neue Fassung vor der alten geprueft wurde -
    und die alte danach einen anderen Fehlertext gab als erwartet. **Ein
    Fehler, der von der Reihenfolge abhaengt, sieht aus wie Zufall.**
    """
    import rustgame_py._sdl2.controller as neu

    return neu


def init():
    """Faehrt das Teilsystem hoch - **dasselbe wie bei `controller`**."""
    _neu().init()


def quit():                                          # noqa: A001
    """Faehrt es herunter."""
    _neu().quit()


def get_init():
    """Laeuft es?"""
    return _neu().get_init()


def get_count():
    """Wie viele Spielgeraete - hier **keines**.

    Dieselbe Antwort wie `_sdl2.controller.get_count()`, und dieselbe wie
    pygame auf einer Maschine ohne Spielgeraet. `minifb` kennt keine
    Spielgeraete; siehe `joystick`.
    """
    return 0


def is_controller(index):
    """Ist das Geraet mit dieser Nummer ein Spielgeraet?"""
    if not isinstance(index, int):
        raise TypeError("an integer is required")
    return False


def name_forindex(index):
    """Der Name zu einer Nummer - `None`, wenn es sie nicht gibt."""
    if not isinstance(index, int):
        raise TypeError("an integer is required")
    return None


def get_eventstate():
    """Werden Spielgeraet-Ereignisse gemeldet?"""
    return _ereignisse[0]


def set_eventstate(state):
    """Schaltet das Melden ein oder aus."""
    _ereignisse[0] = bool(state)
    return None


def update():
    """Fragt die Geraete ab - **gibt es hier nicht zu tun**.

    pygames Beschreibung sagt selbst: *"Will automatically called by the
    event loop, not necessary to call"*. Hier gibt es keine Geraete, also
    auch nichts abzufragen.
    """
    return None


def _nie():
    """Der Fehler, den es hier immer gibt - mit pygames Wortlaut.

    **Zwei Texte, und welcher kommt, haengt an `init()`** - gemessen:
    vor dem Hochfahren `gamecontroller system not initialized`, danach `Index is invalid or not a supported joystick.`.
    Hier stand nur der zweite (bzw. ein selbst erfundener deutscher Satz),
    und damit war der haeufigere Fall - jemand ruft `Controller(0)`, ohne
    vorher `init()` zu rufen - falsch beantwortet.
    """
    import rustgame_py

    if not get_init():
        return rustgame_py.error("gamecontroller system not initialized")
    return rustgame_py.error("Index is invalid or not a supported joystick.")


class Controller:
    """Ein Spielgeraet - **es gibt hier keines.**

    ## Warum die Klasse ihre Namen trotzdem traegt

    Hier stand eine Klasse mit **einem** Namen (`__init__`), und das fiel
    keinem Test auf: `controller_test.py` ueberspringt sich ohne Geraet.
    Gefunden hat es `alle_klassen()` in `api_coverage.py`, nachdem die
    Klassenliste gelesen statt genannt wurde - **14 Namen, keiner da.**

    Gerufen werden kann keiner, denn `Controller(i)` scheitert immer; die
    Namen sind reine Schnittstelle. Ein Programm darf
    `hasattr(Controller, "get_axis")` fragen, und ein Pruefwerkzeug darf
    `dir()` vergleichen.

    ## Die gemessenen Fehlertexte

    Sie haengen am Init-Zustand, und die beiden Fassungen sagen **nicht
    dasselbe**:

    ```text
    vor  init()   error: gamecontroller system not initialized
    nach init()   error: Index is invalid or not a supported joystick.
    Controller()  TypeError: __init__() takes exactly 1 positional argument (0 given)
    Controller(x) TypeError: 'str' object cannot be interpreted as an integer
    ```

    `Controller(True)` ist dabei **keine** Typverletzung: ein `bool` ist
    eine Zahl, und pygame gibt denselben Fehler wie fuer `1`.
    """

    def __init__(self, *args):
        """**`*args`, damit die Saetze stimmen** - siehe die neue Fassung.

        Diese hier sagt `__init__() takes exactly 1 positional argument
        (0 given)`, und sie nimmt ein `0.5` **an**: Cythons `int`-Argument
        schneidet eine Fliesskommazahl ab. Beides gemessen.
        """
        if len(args) != 1:
            raise TypeError(
                "__init__() takes exactly 1 positional argument (%d given)"
                % len(args))
        index = args[0]
        if isinstance(index, (bool, float)):
            # `True` ist 1, und `0.5` wird 0 - beides gemessen.
            index = int(index)
        if not isinstance(index, int):
            raise TypeError("an integer is required")
        raise _nie()

    @property
    def id(self):
        """Die Nummer, mit der das Geraet geoeffnet wurde."""
        raise _nie()

    @property
    def name(self):
        """Der Name des Geraets."""
        raise _nie()

    @staticmethod
    def from_joystick(joystick):
        """Baut eine Steuerung aus einem `pygame.joystick.Joystick`."""
        raise TypeError("should be a pygame.joystick.Joystick object.")

    def as_joystick(self):
        """Gibt dasselbe Geraet als `Joystick`."""
        raise _nie()

    def attached(self):
        """Haengt das Geraet noch dran?"""
        raise _nie()

    def init(self):
        """Oeffnet das Geraet."""
        raise _nie()

    def quit(self):
        """Schliesst es."""
        raise _nie()

    def get_init(self):
        """Ist es offen?"""
        raise _nie()

    def get_axis(self, axis):
        """Eine Achse - Fliesskomma von -1 bis 1."""
        raise _nie()

    def get_button(self, button):
        """Ein Knopf."""
        raise _nie()

    def get_mapping(self):
        """Die Belegungstabelle als Woerterbuch."""
        raise _nie()

    def set_mapping(self, mapping):
        """Setzt die Belegungstabelle."""
        raise _nie()

    def rumble(self, low_frequency, high_frequency, duration):
        """Ruettelt."""
        raise _nie()

    def stop_rumble(self):
        """Hoert auf zu ruetteln."""
        raise _nie()
