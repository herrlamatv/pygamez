"""`pygame.Surface.__init__` - die Groesse, die `__new__` nicht verlangt.

## Die Grenze, die hier gefallen ist

`Surface() ohne Groesse` stand als benannte Grenze: PyO3 bilde `__init__`
nicht auf `tp_init` ab, also seien `Surface()` und
`Surface.__new__(Surface)` nicht zu unterscheiden. Dazu der Satz, der
Ausweg von `PixelArray` und `ftfont` (eine Unterklasse **in Python**) trage
hier nicht, weil die meisten Flaechen in Rust entstehen
(`image.load`, `font.render`, `display.set_mode`, `copy`, `subsurface`) und
dann keine Instanzen der Python-Klasse mehr waeren.

Der zweite Satz ist richtig - und der erste war falsch. Eine PyO3-Klasse
ist ein **Heap-Typ**, und dort setzt `setattr(Klasse, "__init__", f)` den
Schlitz mit: CPythons `type_setattro` ruft `fixup_slot_dispatchers`, und
das legt `tp_init` auf `slot_tp_init`. Es braucht **keine** Unterklasse.

Damit stimmt jetzt beides, und beides ist gemessen:

```text
Surface()                             TypeError: function missing required
                                      argument 'size' (pos 1)
Surface.__new__(S)                    <Surface(Dead Display)>
Surface.__new__(S).get_size()         error: Surface is not initialized
Surface.__new__(S, (4,4)).get_size()  error: Surface is not initialized
Surface((4,4)).get_size()             (4, 4)
```

Die vierte Zeile ist die ueberraschende: pygames `tp_new` **sieht seine
Argumente gar nicht an**.
"""

__all__ = ["__init__"]


#: Die vier Schluesselwoerter, die pygames `surface_init` kennt - in
#: dieser Reihenfolge, aus `Surface((w, h), flags=0, depth=0, masks=None)`.
BEKANNT = ("size", "flags", "depth", "masks")


def __init__(self, *args, **kwargs):
    """Richtet die Flaeche ein - `size` ist Pflicht.

    Der Wortlaut ohne Argument ist pygames, Wort fuer Wort. Mit `*args` und
    nicht mit einem benannten Beiwert, weil Python sonst seinen eigenen
    Satz schreibt (`__init__() missing 1 required positional argument`).

    ## Warum hier zweimal geprueft wird

    Gemessen, und die Reihenfolge ist der Punkt:

    ```text
    Surface(unsinn=1)         TypeError: function missing required argument
                              'size' (pos 1)
    Surface((2, 2), unsinn=1) TypeError: this function got an unexpected
                              keyword argument 'unsinn'
    ```

    pygames Argumentleser zaehlt also **zuerst** die Pflichtstellen und
    sieht sich **danach** die Schluesselwoerter an. Hier wurde beides an
    `_init` weitergereicht, und PyO3 schrieb
    `Surface._init() got an unexpected keyword argument 'unsinn'` - mit
    dem **Innennamen** darin, den pygame nirgends hat. Der Satz fuer ein
    unbekanntes Schluesselwort ist sonst eine benannte Grenze; an dieser
    einen Stelle ist er zu treffen, weil die Funktion ihren Beutel selbst
    nimmt.
    """
    if not args and "size" not in kwargs:
        raise TypeError("function missing required argument 'size' (pos 1)")
    for name in kwargs:
        if name not in BEKANNT:
            raise TypeError(
                "this function got an unexpected keyword argument %r" % name)
    self._init(*args, **kwargs)
