"""`pygame.PixelArray.__init__` - das Eintragen der Sperre.

## Der Umweg, der nicht noetig war

Hier stand eine **Python-Unterklasse** der Rust-Klasse, und die Begruendung
daneben war richtig: PyO3 0.26 kennt keinen `tp_init`-Schlitz, also sind
`PixelArray()` und `PixelArray.__new__(PixelArray)` nicht zu unterscheiden - eine Methode namens
`__init__` steht im Woerterbuch des Typs und wird von `type.__call__` nie
gerufen. Eine Unterklasse in Python hat den Schlitz von selbst.

**Der Schluss daraus war falsch.** Eine PyO3-Klasse ist ein **Heap-Typ**,
und bei einem Heap-Typ setzt `setattr(Klasse, "__init__", f)` den Schlitz
mit: CPythons `type_setattro` ruft `fixup_slot_dispatchers`, und das legt
`tp_init` auf `slot_tp_init`. Nachgemessen - es geht, und zwar an der
echten Klasse.

Gefunden hat den Umweg nicht ein Test, sondern eine Probe, die **die
Basisklassen** vergleicht:

```text
pygame   PixelArray.__mro__[1:]   ('object',)
rustgame PixelArray.__mro__[1:]   ('PixelArray', 'object')
```

Eine Ebene zuviel, und mit ihr ein falsches `__module__`
(`rustgame_py.pixelarray` - das stimmte hier zufaellig`). Beides weg.

## Was die Funktion unten leistet

Genau das, was `__init__` von `__new__` unterscheidet:

```text
PixelArray(s)                   ->  s.get_locks() gibt (ar,)
PixelArray.__new__(PixelArray)   ->  keine Sperre
```

Eine Unterklasse, die `__init__` ueberschreibt und `super()` nicht ruft,
ueberspringt sie - und genau darauf prueft pygames Testsuite
(`surflock_test.test_pxarray_ref` und `surface_test.test_get_locks`).
"""


__all__ = ["__init__"]


def __init__(self, surface):
    """Traegt die fertige Sicht als Sperre in die Flaeche ein.

    Das braucht das **fertige** Python-Objekt, und in `__new__` gibt es das
    noch nicht. Nur der Konstruktor sperrt - eine spaeter abgeleitete Sicht
    (`ar[1:3]`) nicht.
    """
    self._sperre_setzen()
