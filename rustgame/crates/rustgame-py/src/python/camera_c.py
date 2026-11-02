"""`pygame._camera` - die Kamera, eine Schicht tiefer.

pygame hat zwei Kamerastuecke: `_camera` (sein C-Modul, ueber Media
Foundation beziehungsweise V4L2) und `_camera_opencv` (reines Python ueber
`cv2`). `pygame.camera` waehlt zwischen ihnen und reicht durch.

Dieses Modul traegt genau die drei Namen, die `pygame.camera` von seiner
Unterlage braucht - gemessen:

```text
pygame._camera   Camera, colorspace, list_cameras
```

Hier zeigen alle drei auf dasselbe wie in `rustgame_py.camera`: es gibt nur
**ein** Kamerastueck (`nokhwa`), und eine zweite Schicht dazwischen waere
eine Schicht ohne Inhalt.

`_camera_opencv` gibt es hier nicht - und bei pygame auf dieser Maschine
auch nicht: ohne `cv2` scheitert der Import mit
`ModuleNotFoundError: No module named 'cv2'`. **Dieselbe Antwort ist keine
Luecke.**
"""

__all__ = ["Camera", "colorspace", "list_cameras"]


def _kam():
    import rustgame_py.camera

    return rustgame_py.camera


def _satz_setzen():
    """Legt pygames `__doc__` auf die echte `Camera` - einmal."""
    try:
        import rustgame_py

        satz = rustgame_py._klassendoku.get("_camera.Camera")
        if satz is not None:
            _kam().Camera.__doc__ = satz
    except Exception:                                # noqa: BLE001
        pass


def __dir__():
    """`__all__` **und** was das Modul selbst eingefuehrt hat.

    Hier stand `sorted(__all__)` - eine von Hand gepflegte Liste, und
    damit genau die Sorte Messgeraet, an der dieses Projekt schon zehn
    blinde Flecken hatte. Ein `import` am Kopf stand dann im Woerterbuch
    des Moduls, aber nicht in `dir()`.

    `globals()` ist die Antwort, die sich selbst nachfuehrt.
    """
    return sorted(set(__all__) | {n for n in globals() if not n.startswith("_")})


def __getattr__(name):
    """Reicht die drei Namen durch - **die echten**, nicht die Platzhalter.

    `pygame.camera` tauscht `Camera` und `list_cameras` erst bei `init()`
    gegen die echten aus; `pygame._camera` traegt sie von Anfang an.
    Gemessen: `_camera.Camera.__mro__[1:]` ist `('object',)`, waehrend
    `camera.Camera` vor `init()` von `AbstractCamera` erbt.
    """
    if name == "Camera":
        # Wie bei `ftfont.Font`: diese Klasse kommt erst beim Zugriff,
        # und der Satz steht in der erzeugten Tabelle.
        _satz_setzen()
        return _kam()._EchteKamera
    if name == "list_cameras":
        return _kam()._echte_liste
    if name in __all__:
        return getattr(_kam(), name)
    raise AttributeError(
        "module %r has no attribute %r" % (__name__, name)
    )
