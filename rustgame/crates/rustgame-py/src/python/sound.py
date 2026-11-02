"""`pygame.mixer.Sound.__init__` - die eine Zeile, die Rust nicht schreiben kann.

## Der Umweg, der nicht noetig war

Hier stand eine **Python-Unterklasse** der Rust-Klasse, und die Begruendung
daneben war richtig: PyO3 0.26 kennt keinen `tp_init`-Schlitz, also sind
`Sound()` und `Sound.__new__(Sound)` nicht zu unterscheiden - eine Methode namens
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
pygame   Sound.__mro__[1:]   ('object',)
rustgame Sound.__mro__[1:]   ('Sound', 'object')
```

Eine Ebene zuviel, und mit ihr ein falsches `__module__`
(`rustgame_py._sound` statt `rustgame_py.mixer`). Beides weg.

## Was die Funktion unten leistet

Genau das, was `__init__` von `__new__` unterscheidet:

```text
Sound()                 ->  TypeError
Sound.__new__(Sound)    ->  ein leerer Klang
```

Eine Unterklasse, die `__init__` ueberschreibt und `super()` nicht ruft,
ueberspringt sie - und genau darauf prueft pygames Testsuite
(`mixer_test.test_incorrect_subclassing` und `test_sound__without_arg`).
"""


__all__ = ["__init__"]


def __init__(self, *args, **kwargs):
    """Die Pruefung, die **nur** beim gewoehnlichen Aufruf greift.

    `__new__` hat die Werte schon geholt, wenn welche kamen. Eine
    Unterklasse, die diese Zeile ueberspringt, behaelt ihren leeren Klang.
    """
    if len(args) + len(kwargs) != 1:
        raise TypeError("Sound takes either 1 positional or 1 keyword argument")
    # **Nur laden, wenn noch nichts drin ist.** Beim gewoehnlichen
    # `Sound(datei)` hat `__new__` die Datei schon gelesen; ein zweites
    # Lesen waere verschenkte Arbeit. Gebraucht wird der Zweig fuer
    # `super().__init__(file=...)` aus einer Unterklasse und fuer ein
    # `__init__` von Hand nach `Sound.__new__(Sound)`.
    if not self._geladen():
        self._init(*args, **kwargs)
