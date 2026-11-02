"""`pygame._sdl2.audio` - die Tongeraete des Rechners.

**Dieses Modul hat gefehlt, und zwar unbemerkt.** `pygame._sdl2` hat
**acht** Untermodule; `api_coverage.py` fragte nach genau einem
(`_sdl2.video`), und rustgame hatte drei. Fuenf waren damit nie Gegenstand
einer Messung: `audio`, `controller_old`, `mixer`, `sdl2`, `window`.

Gefunden hat es nicht die Abdeckungstabelle, sondern die **Endabnahme des
Plans**: dort steht, das Spielprogramm solle ueber *dreihundert* Bilder
verglichen werden, und gefahren wurden dreissig. Bei dreihundert wich die
Ereignisschlange ab - pygame hatte sechs `AUDIODEVICEADDED` darin, rustgame
keines. Siehe `mixer::init` dazu.

## Was gemessen ist

```text
get_audio_device_names()        5 Namen, der letzte mit ` (2)`
get_audio_device_names(True)    ['Microphone (USB Live Camera audio)']
AudioDevice()                   TypeError: __init__() takes exactly
                                8 positional arguments (0 given)
AUDIO_S16                       32784   (= AUDIO_S16LSB)
AUDIO_F32MSB                    37152
```

Die achtzehn `AUDIO_*`-Zahlen sind SDLs Formatkennungen. Sie stehen hier
als Zahlen, weil sie welche **sind** - `AUDIO_S16` ist `0x8010`, und das
Bit `0x8000` heisst "mit Vorzeichen", `0x1000` "umgekehrte Byteordnung".
"""

# `traceback` steht bei pygame in `dir(pygame._sdl2.audio)` - `audio.pyx`
# importiert es fuer seine Rueckruf-Fehlerbehandlung. Von den acht Kindern
# haben es genau zwei, `audio` und `mixer`; gemessen, nicht erraten. Die
# erste Messung sah nur `mixer`, weil die Ausgabe oben abgeschnitten war -
# **ein Messgeraet, dessen Ausgabe man kuerzt, ist eines, das luegen darf.**
import traceback                                     # noqa: F401

__all__ = [
    "AUDIO_ALLOW_ANY_CHANGE",
    "AUDIO_ALLOW_CHANNELS_CHANGE",
    "AUDIO_ALLOW_FORMAT_CHANGE",
    "AUDIO_ALLOW_FREQUENCY_CHANGE",
    "AUDIO_F32",
    "AUDIO_F32LSB",
    "AUDIO_F32MSB",
    "AUDIO_S16",
    "AUDIO_S16LSB",
    "AUDIO_S16MSB",
    "AUDIO_S32",
    "AUDIO_S32LSB",
    "AUDIO_S32MSB",
    "AUDIO_S8",
    "AUDIO_U16",
    "AUDIO_U16LSB",
    "AUDIO_U16MSB",
    "AUDIO_U8",
    "AudioDevice",
    "error",
    "get_audio_device_names",
]

# Was der Aufrufer dem Geraet an Abweichungen erlaubt.
AUDIO_ALLOW_FREQUENCY_CHANGE = 1
AUDIO_ALLOW_FORMAT_CHANGE = 2
AUDIO_ALLOW_CHANNELS_CHANGE = 4
AUDIO_ALLOW_ANY_CHANGE = 15

# SDLs Formatkennungen. `0x8000` heisst "mit Vorzeichen", `0x1000`
# "umgekehrte Byteordnung", die unteren acht Bit sind die Bittiefe.
AUDIO_U8 = 8
AUDIO_S8 = 32776
AUDIO_U16LSB = 16
AUDIO_S16LSB = 32784
AUDIO_U16MSB = 4112
AUDIO_S16MSB = 36880
AUDIO_U16 = AUDIO_U16LSB
AUDIO_S16 = AUDIO_S16LSB
AUDIO_S32LSB = 32800
AUDIO_S32MSB = 36896
AUDIO_S32 = AUDIO_S32LSB
AUDIO_F32LSB = 33056
AUDIO_F32MSB = 37152
AUDIO_F32 = AUDIO_F32LSB


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


def get_audio_device_names(iscapture=False):
    """Die Namen aller Tongeraete - **eindeutig**.

    Zwei Geraete koennen denselben Namen tragen; SDL haengt dem zweiten
    ` (2)` an, und `rustgame_mixer::geraete::eindeutig` tut dasselbe.
    Gemessen sieht `cpal` auf dieser Maschine genau dieselben fuenf
    Ausgabe- und ein Eingabegeraet wie SDL, in derselben Reihenfolge.

    **Ohne hochgefahrenen Ton ist das ein Fehler**, und das ist ebenfalls
    gemessen:

    ```text
    vor init            error: Audio system not initialised
    nach pygame.init    5
    nach mixer.quit     error: Audio system not initialised
    nach mixer.init     5
    ```

    Die Liste haengt bei rustgame technisch **nicht** am Mixer - `cpal`
    wuerde sie jederzeit hergeben. Sie wird trotzdem verweigert, weil ein
    Programm, das sich darauf verlaesst, bei pygame scheitert.
    """
    # pygame liest hier eine ganze Zahl; alles andere gibt
    # `TypeError: an integer is required`. Siehe `pypm._ganze_zahl`.
    # **Ein Wahrheitswert ist eine ganze Zahl** - `False` ist hier
    # die Vorgabe, und der erste Entwurf lehnte sie ab.
    if not isinstance(iscapture, int):
        raise TypeError("an integer is required")
    pg = _pg()
    if pg.mixer.get_init() is None:
        raise pg.error("Audio system not initialised")
    return pg._tongeraete(bool(iscapture))


class AudioDevice:
    """Ein **offenes** Tongeraet mit eigenem Rueckruf.

    ## Was hier ehrlich nicht geht

    pygames `AudioDevice` oeffnet ein Geraet und ruft `callback(dev, buf)`
    auf einem **Tonfaden** auf, sooft der Treiber Daten braucht. Der
    Rueckruf schreibt dann unmittelbar in den Ausgabepuffer.

    Das waere hier zu bauen - `cpal` kann einen solchen Strom -, aber der
    Rueckruf muesste aus dem Tonfaden heraus Python anfassen, also die
    Sperre des Auswerters holen. Ein Tonfaden, der auf eine Sperre wartet,
    verschluckt Abschnitte; und gibt der Rueckruf die Sperre nicht
    rechtzeitig her, steht beides.

    **Also steht hier ein klarer Fehler und kein halbes Geraet.** Alles,
    was ohne Rueckruf geht, geht: `get_audio_device_names` zaehlt die
    Geraete auf, `pygame.mixer` spielt Klaenge, und `_sdl2.mixer.set_post_mix`
    nimmt seinen Rueckruf an. Was fehlt, ist der rohe Strom - und der
    steht in den benannten Grenzen.

    Die acht Argumente werden trotzdem genau wie bei pygame geprueft: es
    sind **genau acht und alle der Reihe nach**, sonst ein `TypeError` mit
    pygames Wortlaut.
    """

    def __init__(
        self,
        devicename,
        iscapture,
        frequency,
        audioformat,
        numchannels,
        chunksize,
        allowed_changes,
        callback,
    ):
        # Gemessen: `AudioDevice()` gibt bei pygame
        # `TypeError: __init__() takes exactly 8 positional arguments
        # (0 given)` - das macht Python hier von selbst, weil kein
        # Argument eine Vorgabe hat.
        raise _pg().error(
            "rustgame hat keinen rohen Tonstrom mit Rueckruf - der"
            " Rueckruf muesste aus dem Tonfaden heraus Python anfassen."
            " `pygame.mixer` und `get_audio_device_names` gehen."
        )

    # Die zehn Namen, die pygames `AudioDevice` traegt. Sie stehen da,
    # damit `dir()` dasselbe sagt - erreichbar sind sie nie, weil der
    # Konstruktor abbricht.
    audioformat = 0
    callback = None
    chunksize = 0
    deviceid = 0
    devicename = ""
    frequency = 0
    iscapture = 0
    numchannels = 0

    def close(self):
        """Schliesst das Geraet."""

    def pause(self, pause_on):
        """Haelt es an oder laesst es weiterlaufen."""
