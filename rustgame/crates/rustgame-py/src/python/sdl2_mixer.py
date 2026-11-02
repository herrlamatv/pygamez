"""`pygame._sdl2.mixer` - der Haken hinter der Mischstufe.

Ein einziger Aufruf: `set_post_mix(f)` haengt `f` hinter den fertig
gemischten Tonstrom. SDL_mixer ruft ihn dann auf seinem **Tonfaden** auf,
sooft ein Abschnitt fertig ist - gemessen siebenundzwanzig Mal in drei
Zehntelsekunden, jeweils mit `(postmix_objekt, memoryview ueber 2048
Bytes)`.

## Warum das hier nicht geht, und warum ein Fehler die bessere Antwort ist

rustgames Mixer hat **keine zentrale Mischstufe**. Das ist eine
Entscheidung aus dem Plan und sie steht im Kopf von `rustgame-mixer`: jeder
Kanal gibt sein Stueck einzeln an `rodio` weiter, und alles, was von
aussen messbar ist - ob ein Kanal belegt ist, wie lange ein Stueck laeuft,
welche Lautstaerke eingestellt ist -, rechnet der Zustand aus der Uhr aus.
Es gibt also keinen Strom, hinter den sich ein Haken haengen liesse.

Dazu kaeme dasselbe Problem wie bei `_sdl2.audio.AudioDevice`: der Haken
muesste aus einem Tonfaden heraus Python anfassen.

Also nimmt `set_post_mix(None)` seine Arbeit auf (es ist nichts
abzuschalten) und `set_post_mix(f)` sagt klar, dass es den Strom nicht
gibt. Ein Haken, der angenommen und nie gerufen wird, waere die
schlechtere Antwort - ein Programm, das damit ein Pegelbild zeichnet,
saehe eine flache Linie und suchte den Fehler bei sich.

## Nebenbei gemessen: pygame stuerzt hier ab

Ein Rueckruf mit **einem** Argument statt zwei bringt pygame zum
Abstuerzen - kein `TypeError`, ein Speicherzugriffsfehler:

```text
TypeError: f() takes 1 positional argument but 2 were given
   (in pygame/_sdl2/mixer.pyx, recording_cb)
Segmentation fault
```

SDL_mixer ruft den Haken aus C heraus; die Ausnahme hat dort niemanden,
der sie auffaengt. Hier kann das nicht passieren, weil der Haken nie
gerufen wird - aber es ist der Grund, warum diese Grenze keine schmerzt.
"""

# `traceback` steht bei pygame in `dir(pygame._sdl2.mixer)` - und von dort
# aus auch in `dir(pygame._sdl2)`, als einziges der acht Kinder. Gemessen,
# nicht erraten.
import traceback                                     # noqa: F401

__all__ = ["error", "set_post_mix"]


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


def set_post_mix(mix_func):
    """Haengt einen Rueckruf hinter die Mischstufe - **gibt es hier nicht**.

    `None` schaltet ab und geht darum; alles andere gibt einen klaren
    Fehler. Siehe den Modulkopf.
    """
    if mix_func is None:
        return None
    # **Kein `TypeError` fuer etwas, das nicht aufrufbar ist.** Gemessen
    # nimmt pygame auch `set_post_mix(7)` an und sagt nichts - es prueft
    # erst beim Rufen, und dann stuerzt es ab (siehe Modulkopf). Hier
    # scheitert ohnehin jeder Haken, also hat die Unterscheidung keinen
    # Zweck und waere nur eine zweite Abweichung.
    raise _pg().error(
        "rustgame hat keine zentrale Mischstufe, hinter die sich ein"
        " Rueckruf haengen liesse - jeder Kanal geht einzeln an die"
        " Tonkiste. Siehe den Kopf von rustgame-mixer."
    )
