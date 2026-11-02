"""`pygame.pypm` - die Schicht unter `pygame.midi`.

pygames `midi.py` ist eine Huelle um dieses Modul; `pypm` selbst ist die
Bindung an **PortMidi**. Hier liegt es umgekehrt in derselben Richtung: die
Geraetearbeit steckt in `rustgame-mixer/src/midi.rs` (ueber `midir`), und
dieses Modul gibt ihr die Namen, unter denen PortMidi sie anbietet.

Das Modul steht nicht aus Vollstaendigkeitsliebe da. `pygame.midi` **ruft
es auf**, und wer fremden Code uebernimmt, der `from pygame import pypm`
schreibt, soll nicht an einem `ModuleNotFoundError` scheitern.

## Was gemessen ist

```text
Initialize()                  None
Terminate()                   None
CountDevices()                2
GetDeviceInfo(0)              (b'MMSystem', b'Microsoft MIDI Mapper', 0, 1, 0)
GetDeviceInfo(99)             None
GetDefaultInputDeviceID()     -1
GetDefaultOutputDeviceID()    0
Time()                        0
GetErrorText(0)               b''
GetErrorText(-9999)           b"PortMidi: `Invalid device ID'"
GetErrorText(-1)              b"PortMidi: `Illegal error number'"
Channel(1)                    1
Channel(15)                   16384
Channel(0)                    -2147483648
```

## Zwei gemessene Eigenheiten, die man nicht erraten kann

**`Channel(0)` ist `-2147483648`.** PortMidis `Pm_Channel(channel)` ist
`1 << (channel - 1)`, und bei `channel = 0` schiebt C um **-1**. Das ist
undefiniert; auf x86 maskiert der Prozessor die Schiebeweite auf fuenf Bit,
schiebt also um 31, und das Ergebnis `0x80000000` kommt als
vorzeichenbehaftete Zahl heraus. Die Zahl steht hier nicht, weil sie
richtig waere, sondern weil sie **gemessen** ist - wer sie herleitet,
bekommt 0 oder einen Fehler.

**`GetErrorText` kennt nur elf Zahlen.** Alles ausserhalb von -10000 bis
-9992 (und ausser 0) gibt `PortMidi: \\`Illegal error number'` - auch 1,
auch 2, auch -1. Die 0 gibt die **leere** Zeichenkette, nicht "kein
Fehler".

## Die Geraeteanzahl ist eine benannte Grenze

`CountDevices()` gibt bei pygame 2 und hier 1: PortMidi zaehlt den
`Microsoft MIDI Mapper` mit, ein Altlast-Pseudogeraet, das auf das
Standardgeraet zeigt. `midir` zaehlt ihn nicht. Siehe die Grenze
`midi.Geraeteliste`.
"""

# **`array` steht hier, weil es bei pygame hier steht.** PortMidis
# Python-Bindung legt ihre Ereignispuffer in `array.array` ab, und damit ist
# der Name Teil von `dir(pygame.pypm)`. Hier wird er nicht gebraucht - die
# Puffer sind Rusts `Vec` -, aber ein Name zuviel ist derselbe Fehler wie
# einer zuwenig.
import array                                        # noqa: F401
import sys                                          # noqa: F401

__all__ = [
    "Channel",
    "CountDevices",
    "FALSE",
    "FILT_ACTIVE",
    "FILT_AFTERTOUCH",
    "FILT_CHANNEL_AFTERTOUCH",
    "FILT_CLOCK",
    "FILT_CONTROL",
    "FILT_F9",
    "FILT_FD",
    "FILT_MTC",
    "FILT_NOTE",
    "FILT_PITCHBEND",
    "FILT_PLAY",
    "FILT_POLY_AFTERTOUCH",
    "FILT_PROGRAM",
    "FILT_REALTIME",
    "FILT_RESET",
    "FILT_SONG_POSITION",
    "FILT_SONG_SELECT",
    "FILT_SYSEX",
    "FILT_TICK",
    "FILT_TUNE",
    "FILT_UNDEFINED",
    "GetDefaultInputDeviceID",
    "GetDefaultOutputDeviceID",
    "GetDeviceInfo",
    "GetErrorText",
    "Initialize",
    "Input",
    "Output",
    "TRUE",
    "Terminate",
    "Time",
]

FALSE = 0
TRUE = 1

# PortMidis Filterbits (`portmidi.h`). Gemessen, nicht abgeleitet - die
# Gruppen (`FILT_REALTIME`, `FILT_AFTERTOUCH`) sind Summen mehrerer Bits.
FILT_ACTIVE = 1
FILT_SYSEX = 2
FILT_CLOCK = 4
FILT_PLAY = 8
FILT_TICK = 16
FILT_F9 = 16
FILT_FD = 32
FILT_RESET = 64
FILT_REALTIME = 127
FILT_NOTE = 128
FILT_CHANNEL_AFTERTOUCH = 256
FILT_POLY_AFTERTOUCH = 512
FILT_AFTERTOUCH = 768
FILT_PROGRAM = 1024
FILT_CONTROL = 2048
FILT_PITCHBEND = 4096
FILT_MTC = 8192
FILT_SONG_POSITION = 16384
FILT_SONG_SELECT = 32768
FILT_TUNE = 65536
FILT_UNDEFINED = 48

# Die elf Zahlen, die `GetErrorText` kennt - gemessen von -10001 bis 2.
_FEHLER = {
    0: b"",
    -10000: b"PortMidi: `Host error'",
    -9999: b"PortMidi: `Invalid device ID'",
    -9998: b"PortMidi: `Insufficient memory'",
    -9997: b"PortMidi: `Buffer too small'",
    -9996: b"PortMidi: `Buffer overflow'",
    -9995: b"PortMidi: `Bad pointer'",
    -9994: b"PortMidi: `Invalid MIDI message Data'",
    -9993: b"PortMidi: `Internal PortMidi Error'",
    -9992: b"PortMidi: `Buffer cannot be made larger'",
}
_UNBEKANNT = b"PortMidi: `Illegal error number'"


def _pg():
    import rustgame_py

    return rustgame_py


def __dir__():
    """`__all__` **und** was das Modul selbst eingefuehrt hat.

    Vorher stand hier `sorted(__all__)`, und damit war `dir()` eine von
    Hand gepflegte Liste - genau die Sorte, an der dieses Projekt schon
    neun blinde Flecken hatte. Ein `import` am Kopf des Moduls stand dann
    im Woerterbuch, aber nicht in `dir()`.

    `globals()` ist die Antwort, die sich selbst nachfuehrt.
    """
    return sorted(set(__all__) | {n for n in globals() if not n.startswith("_")})


def Initialize():                                    # noqa: N802
    """Faehrt PortMidi hoch - hier **nichts zu tun**.

    `midir` oeffnet Geraete einzeln und braucht keinen Hochlauf. Gemessen
    gibt pygames `Initialize()` ebenfalls `None` und keine Auskunft
    darueber, ob es geklappt hat.
    """
    return None


def Terminate():                                     # noqa: N802
    """Faehrt es herunter - dito."""
    return None


def CountDevices():                                  # noqa: N802
    """Wie viele MIDI-Geraete es gibt.

    Bei pygame 2, hier 1 - siehe die Grenze `midi.Geraeteliste`.
    """
    return _pg()._midi_anzahl()


def _ganze_zahl(wert):
    """`an integer is required` - pygames Satz fuer alles, was keine ist.

    PortMidis Cython-Huelle liest ihre Zahlen mit `PyLong_AsLong`, und
    dessen Einwand lautet genau so. Hier lief stattdessen Pythons
    `int("argname")` durch und gab `ValueError: invalid literal for int()
    with base 10` - andere Art, anderer Satz. Gefunden hat das die Probe
    `Fehlertexte`.
    """
    # Ein Wahrheitswert ist eine ganze Zahl - `int(True)` ist 1, und
    # pygames Cython-Huelle nimmt ihn an.
    if not isinstance(wert, int):
        raise TypeError("an integer is required")
    return wert


def GetDeviceInfo(device_id):                        # noqa: N802
    """Die Auskunft zu einem Geraet - `None` hinter dem Ende.

    `(schnittstelle, name, eingang, ausgang, offen)`, die ersten beiden als
    `bytes`.
    """
    device_id = _ganze_zahl(device_id)
    if device_id < 0:
        return None
    return _pg()._midi_geraet(device_id)


def GetDefaultInputDeviceID():                       # noqa: N802
    """Die Nummer des ersten Eingangs - `-1`, wenn es keinen gibt."""
    return _pg()._midi_erster_eingang()


def GetDefaultOutputDeviceID():                      # noqa: N802
    """Die Nummer des ersten Ausgangs - `-1`, wenn es keinen gibt."""
    return _pg()._midi_erster_ausgang()


def Time():                                          # noqa: N802
    """Die Uhr in Millisekunden."""
    import rustgame_py.midi

    return rustgame_py.midi.time()


def GetErrorText(err_no):                            # noqa: N802
    """Der Text zu einer PortMidi-Fehlernummer, als `bytes`.

    Nur elf Zahlen sind bekannt; alles andere gibt
    `PortMidi: \\`Illegal error number'` - auch `1`, auch `-1`. Die `0`
    gibt die **leere** Zeichenkette.
    """
    return _FEHLER.get(_ganze_zahl(err_no), _UNBEKANNT)


def Channel(channel):                                # noqa: N802
    """Die Bitmaske fuer einen Kanal - `1 << (channel - 1)`.

    **`Channel(0)` ist `-2147483648`**, und das ist gemessen: in C schiebt
    `1 << -1` um eine negative Weite, der Prozessor maskiert sie auf fuenf
    Bit und schiebt um 31. Siehe den Modulkopf.
    """
    # **Kein eigener Einwand.** pygame rechnet hier blind
    # `1 << (chan - 1)`, und eine Zeichenkette gibt darum Pythons
    # `TypeError: unsupported operand type(s) for -: 'str' and 'int'`.
    # Ein `_ganze_zahl(..)` davor waere ein saubererer Satz und ein
    # anderer - gemessen ist dieser.
    n = channel - 1 + 1
    wert = 1 << ((n - 1) & 31)
    # Auf eine vorzeichenbehaftete 32-Bit-Zahl zusammenfalten - genau so
    # kommt die Zahl aus PortMidi heraus.
    if wert >= 1 << 31:
        wert -= 1 << 32
    return wert


class Input:
    """Ein MIDI-Eingang, so wie PortMidi ihn anbietet.

    Fuenf Methoden mit grossen Anfangsbuchstaben - `pygame.midi.Input`
    liegt darueber und bietet dieselbe Sache in pygames Schreibweise an.
    """

    def __init__(self, input_device, buffer_size=4096):
        self._ein = _pg()._MidiEingang(input_device, buffer_size)

    def __setstate__(self, zustand):
        """**Nicht einzupoekeln** - und das ist pygames Wortlaut.

        Gemessen gibt `pygame.pypm.Input().__setstate__({})` dort
        `TypeError: self.midi cannot be converted to a Python object for
        pickling` - Cythons Satz fuer eine Klasse mit einem C-Feld. Der
        Name steht in `dir()`, also steht er hier; der Satz ist
        uebernommen, weil ein eigener an derselben Stelle etwas anderes
        waere.
        """
        raise TypeError(
            "self.midi cannot be converted to a Python object for pickling")

    def Close(self):                                 # noqa: N802
        """Schliesst den Eingang."""
        if self._ein is not None:
            self._ein.schliessen()

    def Poll(self):                                  # noqa: N802
        """Liegt etwas an?"""
        return self._ein.anliegt()

    def Read(self, length):                          # noqa: N802
        """Nimmt bis zu `length` Ereignisse heraus.

        Die Form ist `[[[status, data1, data2, data3], zeit], ...]` -
        **immer vier Datenbytes**, aufgefuellt mit Nullen. `midis2events`
        rechnet damit.
        """
        aus = []
        for roh, zeit in self._ein.lesen(length):
            vier = list(roh[:4]) + [0] * (4 - len(roh[:4]))
            aus.append([vier, zeit])
        return aus

    def SetFilter(self, filters):                    # noqa: N802
        """Welche Nachrichtenarten verworfen werden - **hier keine**.

        PortMidi siebt im Treiber; `midir` gibt alles heraus. Die Maske
        wird angenommen und gemerkt, damit ein Programm nicht scheitert -
        und sie steht in den benannten Grenzen, damit niemand glaubt, sie
        wirke.
        """
        self._filter = int(filters)
        return None

    def SetChannelMask(self, mask):                  # noqa: N802
        """Welche Kanaele durchkommen - dito."""
        self._kanalmaske = int(mask)
        return None


class Output:
    """Ein MIDI-Ausgang, so wie PortMidi ihn anbietet."""

    def __init__(self, output_device, latency=0, buffer_size=256):
        self._aus = _pg()._MidiAusgang(output_device)
        self._latency = latency

    def __setstate__(self, zustand):
        """Nicht einzupoekeln - siehe `Input.__setstate__`."""
        raise TypeError(
            "self.midi cannot be converted to a Python object for pickling")

    def Close(self):                                 # noqa: N802
        """Schliesst den Ausgang."""
        if self._aus is not None:
            self._aus.schliessen()

    def Abort(self):                                 # noqa: N802
        """Bricht Ausstehendes ab.

        Hier ist nichts ausstehend - `midir` sendet sofort. Es bleibt der
        Teil, den pygame ebenfalls tut: `All Sound Off` auf allen sechzehn
        Kanaelen.
        """
        for kanal in range(16):
            self._aus.senden(bytes([0xB0 | kanal, 120, 0]))

    def Write(self, data):                           # noqa: N802
        """Schreibt eine Liste von Ereignissen."""
        import rustgame_py.midi as _m

        for eintrag in data:
            nachricht = eintrag[0] if isinstance(eintrag, (list, tuple)) else eintrag
            bytes_ = [int(b) & 0xFF for b in nachricht]
            if not bytes_:
                continue
            self._aus.senden(bytes(bytes_[: _m._laenge(bytes_[0], len(bytes_))]))

    def WriteShort(self, status, data1=0, data2=0):  # noqa: N802
        """Schreibt **eine** kurze Nachricht."""
        import rustgame_py.midi as _m

        roh = [int(status) & 0xFF, int(data1) & 0x7F, int(data2) & 0x7F]
        self._aus.senden(bytes(roh[: _m._laenge(roh[0], 3)]))

    def WriteSysEx(self, when, msg):                 # noqa: N802
        """Schreibt eine Systemnachricht - `msg` als Liste oder Text."""
        if isinstance(msg, str):
            roh = [ord(c) & 0xFF for c in msg]
        else:
            roh = [int(b) & 0xFF for b in msg]
        self._aus.senden(bytes(roh))
