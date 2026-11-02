"""Ein `__init__`, das nur weiterreicht - fuer `tp_new` gegen `tp_init`.

## Wozu

pygame trennt beide Schritte: `tp_new` legt das Objekt im **Nullzustand**
an und sieht seine Argumente gar nicht an, `tp_init` liest sie. Gemessen
an `pygame.geometry`:

```text
Circle.__new__(Circle)         Circle((0.0, 0.0), 1.0)
Circle.__new__(Circle, 1,2,3)  dasselbe - die Argumente zaehlen nicht
Circle()                       TypeError: Arguments must be a Circle, …
c.__init__(1, 2, 3)            Circle((1.0, 2.0), 3.0)
Line.__new__(Line)             Line((0.0, 0.0), (0.0, 0.0))
```

PyO3 kennt nur `#[new]`, und dort kommen beide Aufrufe gleich an. Der
Ausweg ist derselbe wie bei `Surface` und `Color`: eine PyO3-Klasse ist
ein **Heap-Typ**, und `setattr(Klasse, "__init__", f)` setzt `tp_init`
mit (CPythons `type_setattro` ruft `fixup_slot_dispatchers`). `#[new]`
gibt seitdem den Nullzustand, sobald niemand etwas uebergibt, und dieses
`__init__` reicht die Argumente an `_init` weiter - **auch das leere
Nichts**, denn der Einwand gegen `Circle()` steht in Rust und soll dort
bleiben.

Eine Datei fuer alle Klassen dieser Art: der Satz, der bei falschen
Argumenten faellt, haengt an `_init` und nicht hier.
"""

__all__ = ["__init__"]


def __init__(self, *args, **kwargs):
    """Reicht alles an `_init` weiter - der prueft und meldet."""
    self._init(*args, **kwargs)
