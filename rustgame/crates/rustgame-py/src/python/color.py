"""`pygame.Color.__init__` - der Fehler, den `__new__` nicht gibt.

## Die Grenze, die hier gefallen ist

`Color()` stand als benannte Grenze: PyO3 kenne nur `#[new]`, und
`Color()` und `Color.__new__(Color)` kaemen dort beide mit einem leeren
Argumentbeutel an, also seien sie nicht zu trennen. Der Vorzug ging an
`__new__`, weil pygames `test_new` es prueft - und `Color()` gab darum
eine schwarze Farbe statt eines `ValueError`.

Zu trennen sind sie doch, auf demselben Weg wie bei `Surface`: eine
PyO3-Klasse ist ein **Heap-Typ**, und `setattr(Klasse, "__init__", f)`
setzt dort `tp_init` mit (CPythons `type_setattro` ruft
`fixup_slot_dispatchers`). `__new__` legt weiter die schwarze Farbe an,
dieses `__init__` liest die Argumente - und damit stimmt jede der sechs
gemessenen Zeilen:

```text
Color.__new__(Color)          Color(0, 0, 0, 255)
Color.__new__(Color, 1, 2, 3) Color(0, 0, 0, 255)   - sieht nichts an
Color()                       ValueError: invalid color (…)
Color(unsinn=1)               dasselbe              - Schluesselwoerter zaehlen nicht
Color(1, 2, 3, unsinn=1)      Color(1, 2, 3, 255)   - und werden uebergangen
c.__init__()                  ValueError: invalid color (…)
```

Die vierte und fuenfte Zeile sind der Grund, warum hier **nur** die
Stellungsargumente gezaehlt werden: pygames `tp_init` liest sie mit
`PyArg_ParseTuple`, das die Schluesselwoerter gar nicht ansieht.
"""

__all__ = ["__init__"]

#: Der Satz, mit dem pygame eine unbrauchbare Farbe ablehnt - Wort fuer
#: Wort, und derselbe wie in `farbe.rs`.
FEHLER = ("invalid color (color sequence must have size 3 or 4, and each"
          " element must be an integer in the range [0, 255])")


def __init__(self, *args, **kwargs):
    """Setzt die Farbe - ohne Stellungsargument ein `ValueError`.

    Mit `*args` und nicht mit benannten Beiwerten, weil Python sonst
    seinen eigenen Satz schreibt; `kwargs` wird gelesen und **nicht
    benutzt**, genau wie bei pygame.
    """
    if not args:
        raise ValueError(FEHLER)
    self._init(*args)
