"""`pygame.macosx` - der eine Aufruf, den macOS braucht.

pygame fuehrt dieses Modul auf **jeder** Plattform mit; unter Windows und
Linux tut `Video_AutoInit()` nichts und gibt `True`. Gemessen auf dieser
Maschine:

```text
pygame.macosx.Video_AutoInit()   True
```

Unter macOS holt es dort den Hauptfaden an die Fensterverwaltung heran -
etwas, das es ohne Cocoa nicht zu tun gibt. rustgame kommt ohne SDL aus;
`minifb` regelt das selbst.

Das Modul steht trotzdem da, weil `dir(pygame)` es zeigt und weil ein
Programm, das `pygame.macosx.Video_AutoInit()` ruft, auf **allen**
Plattformen durchkommen soll - genau das ist der Zweck der Funktion.
"""

# **Drei mitimportierte Namen.** `pygame/macosx.py` schreibt
# `import os`, `import platform` und `import sys` - und damit stehen sie in
# `dir(pygame.macosx)`. Gebraucht werden sie hier nicht (es gibt nichts zu
# tun ausserhalb von macOS), aber ein Name zuviel ist derselbe Fehler wie
# einer zuwenig.
import os                                            # noqa: F401
import platform                                      # noqa: F401
import sys                                           # noqa: F401

__all__ = ["Video_AutoInit"]


def __dir__():
    """`__all__` **und** was das Modul selbst eingefuehrt hat.

    Hier stand `sorted(__all__)` - eine von Hand gepflegte Liste, und
    damit genau die Sorte Messgeraet, an der dieses Projekt schon zehn
    blinde Flecken hatte. Ein `import` am Kopf stand dann im Woerterbuch
    des Moduls, aber nicht in `dir()`.

    `globals()` ist die Antwort, die sich selbst nachfuehrt.
    """
    return sorted(set(__all__) | {n for n in globals() if not n.startswith("_")})


def Video_AutoInit():                                # noqa: N802
    """Gibt `True` - gemessen gibt pygame hier ausserhalb von macOS dasselbe."""
    return True
