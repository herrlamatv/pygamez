"""`pygame.__pyinstaller` - der Haken fuer PyInstaller.

PyInstaller sucht in jedem Paket nach `__pyinstaller.get_hook_dirs()` und
liest dort die Haken-Dateien ein, die sagen, welche **Daten** mitgepackt
werden muessen. Bei pygame sind das die Schriften, das Programmsinnbild und
die mitgelieferten Beispieldaten.

Gemessen:

```text
pygame.__pyinstaller.get_hook_dirs()
    ['.../site-packages/pygame/__pyinstaller']
```

## Warum das kein Beiwerk ist

Ohne diesen Haken fehlt einem gepackten Programm `freesansbold.ttf`, und es
stuerzt beim ersten `pygame.font.Font(None, 24)` ab - **nicht** beim
Entwickeln, sondern erst beim Ausliefern. Das ist die unangenehmste Sorte
Fehler, und sie kostet hier zwei Zeilen.

Der Ordner, der zurueckgegeben wird, liegt neben diesem Modul. Welche
Dateien darin stehen muessen, ist Sache von PyInstaller; rustgame
mitgelieferte Dateien sind `freesansbold.ttf` und `pygame_icon.bmp`, und
die liegen beide neben der Huelle - derselbe Ordner, den
`rustgame_font::set_schriftordner` beim ersten Schriftzugriff bekommt.
"""

import os

__all__ = ["get_hook_dirs"]


def __dir__():
    """`__all__` **und** was das Modul selbst eingefuehrt hat.

    Hier stand `sorted(__all__)` - eine von Hand gepflegte Liste, und
    damit genau die Sorte Messgeraet, an der dieses Projekt schon zehn
    blinde Flecken hatte. Ein `import` am Kopf stand dann im Woerterbuch
    des Moduls, aber nicht in `dir()`.

    `globals()` ist die Antwort, die sich selbst nachfuehrt.
    """
    return sorted(set(__all__) | {n for n in globals() if not n.startswith("_")})


def get_hook_dirs():
    """Die Ordner mit den PyInstaller-Haken - hier **einer**."""
    import rustgame_py

    return [os.path.join(os.path.dirname(rustgame_py.__file__), "__pyinstaller")]
