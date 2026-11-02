"""Die fuenf Zuweisungsoperatoren der Vektoren - `v += w` **an Ort**.

## Was ohne sie geschah

Ohne `__iadd__` faellt Python auf `v = v + w` zurueck: das Ergebnis stimmt,
aber es ist ein **neues Objekt**. Gemessen:

```text
v = Vector2(1, 2); i = id(v); v += Vector2(1, 1)
pygame    id(v) == i   True     -  derselbe Vektor
rustgame  id(v) == i   False    -  ein neuer
```

Das ist keine Kosmetik: wer den Vektor woanders festhaelt - ein Sprite,
eine Liste, ein zweiter Name - sieht die Aenderung bei pygame und bei
rustgame nicht.

Gefunden hat es die Probe `Klassendoppel`. **Keine** der 804 anderen
Proben fragte danach, denn sie lassen alle Namen mit Unterstrich aus -
und Doppelstriche sind Namen mit Unterstrich.

## Warum Python und nicht Rust

PyO3 bildet einen Zuweisungsoperator auf `nb_inplace_*` ab und verlangt
dafuer den Rueckgabetyp `()`. Damit laesst sich **kein `NotImplemented`**
zurueckgeben - und genau das gibt pygame fuer einen Operanden, der nicht
passt:

```text
v.__iadd__(3)            NotImplemented
v.__iadd__((1,))         NotImplemented   -  falsche Laenge
v.__imul__(Vector2(2,2)) NotImplemented   -  Python rechnet dann
                                             v = v * w, das Skalarprodukt,
                                             und aus dem Vektor wird eine Zahl
v.__imul__(True)         (4.0, 6.0)       -  True ist 1
v.__itruediv__(0)        ZeroDivisionError: division by zero
```

Ein `TypeError` von hier waere ein anderer Satz an einer anderen Stelle.
Also stehen die fuenf in Python und werden mit
`setattr(Vector2, "__iadd__", f)` angehaengt - bei einem **Heap-Typ** setzt
das den Schlitz mit, dieselbe Einsicht, die `Surface.__init__` und
`Sound.__init__` hierher gebracht hat.

## `//` rechnet anders, als man schreibt

`floor(x * (1/f))` und nicht `floor(x / f)` - dieselbe Reihenfolge wie
`nb_floor_divide` in pygames `math.c`. Bei `f = 3` und `x = 6` ist das der
Unterschied zwischen 1 und 2.
"""

import math

__all__ = ["__iadd__", "__isub__", "__imul__", "__itruediv__", "__ifloordiv__"]


def _achsen(selbst, other):
    """Die Zahlen des anderen Operanden - oder `None`.

    `None` heisst `NotImplemented`: falsche Laenge, keine Folge, oder ein
    Eintrag, der keine Zahl ist.
    """
    try:
        werte = list(other)
    except TypeError:
        return None
    if len(werte) != len(selbst):
        return None
    try:
        return [float(x) for x in werte]
    except (TypeError, ValueError):
        return None


def _zahl(wert):
    """Eine einzelne Zahl - oder `None` fuer `NotImplemented`.

    Eine **Folge** gibt hier `None`, auch wenn sie sich in eine Zahl
    umrechnen liesse: `v *= Vector2(2, 2)` soll zum Skalarprodukt fallen,
    nicht achsenweise rechnen.
    """
    if hasattr(wert, "__len__") or hasattr(wert, "__getitem__"):
        return None
    try:
        return float(wert)
    except (TypeError, ValueError):
        return None


def __iadd__(self, other):
    """`v += w` - an Ort."""
    o = _achsen(self, other)
    if o is None:
        return NotImplemented
    for i, x in enumerate(o):
        self[i] = self[i] + x
    return self


def __isub__(self, other):
    """`v -= w` - an Ort."""
    o = _achsen(self, other)
    if o is None:
        return NotImplemented
    for i, x in enumerate(o):
        self[i] = self[i] - x
    return self


def __imul__(self, wert):
    """`v *= 3` - an Ort, und **nur** mit einer Zahl."""
    f = _zahl(wert)
    if f is None:
        return NotImplemented
    for i in range(len(self)):
        self[i] = self[i] * f
    return self


def __itruediv__(self, wert):
    """`v /= 2` - an Ort."""
    f = _zahl(wert)
    if f is None:
        return NotImplemented
    if f == 0.0:
        raise ZeroDivisionError("division by zero")
    for i in range(len(self)):
        self[i] = self[i] / f
    return self


def __ifloordiv__(self, wert):
    """`v //= 2` - an Ort, mit `floor(x * (1/f))`.

    Die Reihenfolge ist pygames, siehe der Modulkopf.
    """
    f = _zahl(wert)
    if f is None:
        return NotImplemented
    if f == 0.0:
        raise ZeroDivisionError("division by zero")
    kehr = 1.0 / f
    for i in range(len(self)):
        self[i] = math.floor(self[i] * kehr)
    return self
