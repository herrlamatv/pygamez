"""`pygame._data_classes` - wo `PowerState` wohnt.

Zwei Namen, und beide sind gemessen:

```text
PowerState   die Klasse, die `pygame.system.get_power_state()` zurueckgibt
dataclass    `dataclasses.dataclass` - ein **mitimportierter** Name
```

Der zweite ist kein Teil der Schnittstelle, sondern ein Import, der in
`dir()` mitlaeuft. Er steht hier trotzdem, und zwar aus demselben Grund,
aus dem `_sdl2.video` seinen `messagebox` **verloren** hat: was `dir()`
zeigt, ist die Schnittstelle, die ein Programm sieht. Ein Name zuviel und
einer zuwenig sind derselbe Fehler.

`PowerState` selbst ist hier eine Rust-Klasse (`rechner.rs`) und keine
`@dataclass` - von aussen sieht man denselben `repr`, dieselben acht
Felder und dieselbe Unveraenderlichkeit.
"""

from dataclasses import dataclass                    # noqa: F401

__all__ = ["PowerState", "dataclass"]


def _pg():
    import rustgame_py

    return rustgame_py


def __dir__():
    """`__all__` **und** was das Modul selbst eingefuehrt hat.

    Hier stand `sorted(__all__)` - eine von Hand gepflegte Liste, und
    damit genau die Sorte Messgeraet, an der dieses Projekt schon zehn
    blinde Flecken hatte. Ein `import` am Kopf stand dann im Woerterbuch
    des Moduls, aber nicht in `dir()`.

    `globals()` ist die Antwort, die sich selbst nachfuehrt.
    """
    return sorted(set(__all__) | {n for n in globals() if not n.startswith("_")})


@dataclass(frozen=True)
class PowerState:
    """Was `pygame.system.get_power_state()` zurueckgibt.

    **Zeile fuer Zeile pygames eigene Fassung** - acht Felder mit ihren
    Anmerkungen, `frozen=True`, nichts weiter. Hier stand stattdessen eine
    Rust-Klasse, die das `repr` nachbaute; das sah richtig aus und war in
    zwei Dingen anders:

    * **Sechs Doppelstriche fehlten** - `__match_args__` (und damit jedes
      `match`-Muster), `__dataclass_fields__`, `__dataclass_params__`,
      `__replace__` und die zwei Anmerkungs-Felder von Python 3.14.
    * **Acht Namen waren zuviel.** PyO3s `get_all` legt die Felder als
      Klassenbeschreiber ab; `dir(pygame.system.PowerState)` hat dagegen
      **keinen** oeffentlichen Namen, weil ein `@dataclass` ohne `slots`
      die Felder nur an der Instanz hat.

    Gefunden hat das die Probe `Klassendoppel` zusammen mit
    `Klassennamen`. Gemessen gibt pygame auf einem Rechner ohne Akku:

    ```text
    PowerState(battery_percent=None, battery_seconds=None, on_battery=False,
               no_battery=True, charging=False, charged=False,
               plugged_in=True, has_battery=False)
    ```

    Dass die Klasse hier in Python stehen darf, liegt an einer Messung:
    `get_power_state()` gibt in rustgame **immer `None`** - sie wird von
    Rust aus nie gebaut.
    """

    battery_percent: int | None
    battery_seconds: int | None
    on_battery: bool
    no_battery: bool
    charging: bool
    charged: bool
    plugged_in: bool
    has_battery: bool
